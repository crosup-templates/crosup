use anyhow::Error;
use crosup_installers::{
    apk::ApkInstaller, apt::AptInstaller, brew::BrewInstaller, curl::CurlInstaller,
    dnf::DnfInstaller, emerge::EmergeInstaller, git::GitInstaller, nix::NixInstaller,
    pacman::PacmanInstaller, yum::YumInstaller, zypper::ZypperInstaller, Installer,
};
use crosup_macros::{
    add_vertex, add_vertex_with_condition, convert_generic_installer, downcast_installer,
};
use crosup_types::{
    configuration::Configuration,
    curl::{default_brew_installer, default_nix_installer},
};
use os_release::OsRelease;
use owo_colors::OwoColorize;

#[derive(Debug, Clone)]
pub struct Vertex {
    name: String,
    dependencies: Vec<String>,
    provider: String,
    apt: Option<AptInstaller>,
    brew: Option<BrewInstaller>,
    curl: Option<CurlInstaller>,
    git: Option<GitInstaller>,
    nix: Option<NixInstaller>,
    yum: Option<YumInstaller>,
    dnf: Option<DnfInstaller>,
    zypper: Option<ZypperInstaller>,
    apk: Option<ApkInstaller>,
    pacman: Option<PacmanInstaller>,
    emerge: Option<EmergeInstaller>,
}

impl From<Box<dyn Installer + 'static>> for Vertex {
    fn from(installer: Box<dyn Installer + 'static>) -> Self {
        Self {
            name: installer.name().to_string(),
            dependencies: installer
                .dependencies()
                .iter()
                .map(|x| x.to_string())
                .collect(),
            provider: installer.provider().to_string(),
            apt: downcast_installer!("apt", installer, AptInstaller),
            brew: downcast_installer!("brew", installer, BrewInstaller),
            curl: downcast_installer!("curl", installer, CurlInstaller),
            git: downcast_installer!("git", installer, GitInstaller),
            nix: downcast_installer!("nix", installer, NixInstaller),
            yum: downcast_installer!("yum", installer, YumInstaller),
            dnf: downcast_installer!("dnf", installer, DnfInstaller),
            zypper: downcast_installer!("zypper", installer, ZypperInstaller),
            apk: downcast_installer!("apk", installer, ApkInstaller),
            pacman: downcast_installer!("pacman", installer, PacmanInstaller),
            emerge: downcast_installer!("emerge", installer, EmergeInstaller),
        }
    }
}

impl Into<Box<dyn Installer>> for Vertex {
    fn into(self) -> Box<dyn Installer> {
        match self.provider.as_str() {
            "apt" => Box::new(self.apt.unwrap()),
            "brew" => Box::new(self.brew.unwrap()),
            "curl" => Box::new(self.curl.unwrap()),
            "git" => Box::new(self.git.unwrap()),
            "nix" => Box::new(self.nix.unwrap()),
            "yum" => Box::new(self.yum.unwrap()),
            "dnf" => Box::new(self.dnf.unwrap()),
            "zypper" => Box::new(self.zypper.unwrap()),
            "apk" => Box::new(self.apk.unwrap()),
            "pacman" => Box::new(self.pacman.unwrap()),
            "emerge" => Box::new(self.emerge.unwrap()),
            _ => panic!("Unknown installer: {}", self.name),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Edge {
    from: usize,
    to: usize,
}

#[derive(Clone, Debug)]
pub struct InstallerGraph {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
}

impl Into<Vec<Box<dyn Installer>>> for InstallerGraph {
    fn into(self) -> Vec<Box<dyn Installer>> {
        self.vertices.into_iter().map(|x| x.into()).collect()
    }
}

pub fn autodetect_installer(config: &mut Configuration) {
    if let Some(generic_install) = &config.install {
        // detect linux
        if cfg!(target_os = "linux") {
            // determine linux distribution using os-release
            if let Ok(os_release) = OsRelease::new() {
                let os = os_release.id.to_lowercase();
                let os = os.as_str();

                let package_manager = match os {
                    "ubuntu" | "debian" | "linuxmint" | "pop" | "elementary" | "zorin" => {
                        convert_generic_installer!(config, generic_install, apt);
                        "apt-get"
                    }
                    "fedora" | "centos" | "rhel" | "rocky" | "amazon" => {
                        convert_generic_installer!(config, generic_install, dnf);
                        "dnf"
                    }
                    "opensuse" | "sles" => {
                        convert_generic_installer!(config, generic_install, zypper);
                        "zypper"
                    }
                    "arch" | "manjaro" => {
                        convert_generic_installer!(config, generic_install, pacman);
                        "pacman"
                    }
                    "gentoo" => {
                        convert_generic_installer!(config, generic_install, emerge);
                        "emerge"
                    }
                    "alpine" => {
                        convert_generic_installer!(config, generic_install, apk);
                        "apk"
                    }
                    _ => panic!("Unsupported OS: {}", os),
                };

                let os_pretty = os_release.pretty_name;
                println!("-> Detected OS:🐧 {}", os_pretty.magenta());
                println!(
                    "-> Using package manager: 📦 {}",
                    package_manager.bright_green()
                );
            }
        }
        if cfg!(target_os = "macos") {
            println!("-> Detected OS: 🍎 macOS");
            println!("-> Using package manager: 📦 {}", "brew".bright_green());
            convert_generic_installer!(config, generic_install, brew);
        }
    }
}

pub fn build_installer_graph(
    config: &mut Configuration,
) -> (InstallerGraph, Vec<Box<dyn Installer>>) {
    let mut graph = InstallerGraph::new();

    if config.clone().nix.is_some() {
        if let Some(curl) = config.clone().curl {
            if !curl.into_iter().any(|(_, y)| y.script.contains_key("nix")) {
                let nix = default_nix_installer();
                graph.add_vertex(Vertex::from(Box::new(CurlInstaller {
                    name: nix.name.clone(),
                    ..CurlInstaller::from(nix.clone())
                }) as Box<dyn Installer>));
            }
        }
    }

    if config.clone().brew.is_some() {
        if let Some(curl) = config.clone().curl {
            if !curl.into_iter().any(|(_, y)| y.script.contains_key("brew")) {
                let brew = default_brew_installer();
                graph.add_vertex(Vertex::from(Box::new(CurlInstaller {
                    name: brew.name.clone(),
                    ..CurlInstaller::from(brew.clone())
                }) as Box<dyn Installer>));
            }
        }
    }

    autodetect_installer(config);

    add_vertex!(graph, AptInstaller, config, apt, pkg);
    add_vertex!(graph, CurlInstaller, config, curl, script);
    add_vertex!(graph, GitInstaller, config, git, repo);
    add_vertex!(graph, NixInstaller, config, nix, pkg);
    add_vertex!(graph, YumInstaller, config, yum, pkg);
    add_vertex!(graph, DnfInstaller, config, dnf, pkg);
    add_vertex!(graph, ZypperInstaller, config, zypper, pkg);
    add_vertex!(graph, ApkInstaller, config, apk, pkg);
    add_vertex!(graph, PacmanInstaller, config, pacman, pkg);
    add_vertex!(graph, EmergeInstaller, config, emerge, pkg);
    add_vertex_with_condition!(graph, BrewInstaller, config, brew, pkg);

    setup_dependencies(&mut graph);

    (graph.clone(), graph.into())
}

fn setup_dependencies(graph: &mut InstallerGraph) {
    let mut edges = vec![];

    for (i, vertex) in graph.vertices.iter().enumerate() {
        for dependency in vertex.dependencies.iter() {
            if let Some(j) = graph.vertices.iter().position(|x| x.name == *dependency) {
                edges.push(Edge { from: i, to: j });
            }
        }
    }

    graph.edges = edges;
}

impl InstallerGraph {
    pub fn new() -> Self {
        Self {
            vertices: vec![],
            edges: vec![],
        }
    }

    pub fn add_vertex(&mut self, vertex: Vertex) -> usize {
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.push(Edge { from, to });
    }

    pub fn contains(&self, name: &str) -> bool {
        self.vertices.iter().any(|x| x.name == name)
    }

    pub fn install_all(&self) -> Result<(), Error> {
        let mut visited = vec![false; self.vertices.len()];

        for (index, vertex) in self.vertices.iter().enumerate() {
            if !visited[index] {
                self.install(vertex.clone().into(), &mut visited)?;
            }
        }

        Ok(())
    }

    pub fn install(
        &self,
        package: Box<dyn Installer>,
        visited: &mut Vec<bool>,
    ) -> Result<(), Error> {
        let index = self
            .vertices
            .iter()
            .position(|x| x.name == package.name())
            .unwrap();

        if visited[index] {
            return Ok(());
        }

        for edge in self.edges.iter().filter(|x| x.from == index) {
            self.install(self.vertices[edge.to].clone().into(), visited)?;
        }

        package.install()?;

        visited[index] = true;

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.vertices.len()
    }
}