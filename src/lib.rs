use std::io::Write;
use std::path::Path;

#[derive(Debug)]
pub enum ReadError<F: std::str::FromStr> {
    Io(std::io::Error),
    Parse(F::Err),
}

fn read<P: AsRef<Path>, F: std::str::FromStr>(path: P) -> std::result::Result<F, ReadError<F>> {
    let str = std::fs::read_to_string(path).map_err(|e| ReadError::Io(e))?;
    let str = str.trim();
    str.parse::<F>().map_err(|e| ReadError::Parse(e))
}

#[derive(Clone)]
pub struct Namespace {
    path: std::path::PathBuf
}

impl std::fmt::Debug for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Namespace")
        .field("device_path", &self.device_path().map_err(|_| std::fmt::Error)?)
        .field("device_uuid", &self.device_uuid().map_err(|_| std::fmt::Error)?)
        .field("device_nguid", &self.device_nguid().map_err(|_| std::fmt::Error)?)
        .field("enable", &self.enable().map_err(|_| std::fmt::Error)?)
        .finish()
    }
}

impl Namespace {

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn set_enable(&mut self, value: bool) -> std::io::Result<()> {
        let enable_path = self.path().join("enable");
        let mut file = std::fs::File::create(enable_path)?;
        let value_bytes: &[u8; 2] = match value {
            true => b"1\n",
            false => b"0\n",
        };
        file.write_all(value_bytes)?;
        Ok(())
    }

    pub fn enable(&self) -> std::result::Result<bool, ReadError<u8>> {
        read(self.path().join("enable")).map(|v| v == 1)
    }

    pub fn set_ana_grpid(&mut self, value: u32) -> std::io::Result<()> {
        let attr_path = self.path().join("ana_grpid");
        let mut file = std::fs::File::create(attr_path)?;
        let value_string =  value.to_string() + "\n";
        file.write_all(value_string.as_bytes())?;
        Ok(())
    }

    pub fn ana_grpid(&self) -> std::io::Result<u32> {
        let ana_grpid_path = self.path().join("ana_grpid");
        let ana_grpid_str = std::fs::read_to_string(ana_grpid_path).unwrap();
        let ana_grpid = ana_grpid_str.trim_end_matches('\n').parse::<u32>().unwrap();
        Ok(ana_grpid)
    }

    pub fn set_device_nguid(&mut self, value: &str) -> std::io::Result<()> {
        let path = self.path().join("device_nguid");
        let mut file = std::fs::File::create(path)?;
        let value_string = value.to_string() + "\n";
        file.write_all(value_string.as_bytes())?;
        Ok(())
    }

    pub fn device_nguid(&self)-> std::result::Result<String, ReadError<String>> {
        read(self.path().join("device_nguid"))
    }

    pub fn set_device_uuid(&mut self, value: &str) -> std::io::Result<()> {
        let path = self.path().join("device_uuid");
        let mut file = std::fs::File::create(path)?;
        let value_string = value.to_string() + "\n";
        file.write_all(value_string.as_bytes())?;
        Ok(())
    }

    pub fn device_uuid(&self) -> std::result::Result<String, ReadError<String>> {
        read(self.path().join("device_uuid"))
    }

    pub fn set_device_path(&mut self, value: &str) -> std::io::Result<()> {
        let path = self.path().join("device_path");
        let mut file = std::fs::File::create(path)?;
        let value_string = value.to_string() + "\n";
        file.write_all(value_string.as_bytes())?;
        Ok(())
    }

    pub fn device_path(&self) -> std::io::Result<Option<String>> {
        let path = self.path().join("device_path");
        match std::fs::read_to_string(path).unwrap().as_str() {
            "(null)\n" => Ok(None),
            str =>  Ok(Some(str.trim_end_matches('\n').to_string()))
        }
       
    }

}

pub struct Subsystem {
    nqn: std::ffi::OsString
}

const CONFIGFS_DIR: &str = "/sys/kernel/config/nvmet/";

impl std::fmt::Debug for Subsystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subsystem")
        .field("nqn", &self.nqn)
        .field("attr_allow_any_host", &self.attr_allow_any_host().map_err(|_| std::fmt::Error)?)
        .field("attr_cntlid_min", &self.attr_cntlid_min().map_err(|_| std::fmt::Error)?)
        .field("attr_cntlid_max", &self.attr_cntlid_max().map_err(|_| std::fmt::Error)?)
        .field("attr_model", &self.attr_model().map_err(|_| std::fmt::Error)?)
        .field("attr_serial", &self.attr_serial().map_err(|_| std::fmt::Error)?)
        .field("namespaces", &match self.namespaces() {
            Ok(iter) => Ok(iter.collect::<Vec<Namespace>>()),
            Err(_) => Err(std::fmt::Error),
        }?)
        .finish()
    }
}

impl Subsystem {
    /// Add a new subsystem with the given nqn. Return an error if a subsystem with the given
    /// nqn already exists.
    pub fn new<T>(nqn: T) -> std::io::Result<Subsystem> 
    where 
        std::ffi::OsString: From<T>
    {
        let subsys = Subsystem{ nqn: std::ffi::OsString::from(nqn) };
        std::fs::create_dir(subsys.path())?;
        Ok(subsys)
    }

    /// Return the subsystem with the given nqn. This will not return an error if the subsystem does
    /// not exist.
    pub fn open<T>(nqn: T) -> Subsystem 
    where 
        std::ffi::OsString: From<T>
    {
        Subsystem{ nqn: std::ffi::OsString::from(nqn) }
    }

    /// Return a boolean indicating whether or not a subsystem with the given nqn exists.
    pub fn exists<T>(nqn: T) -> std::io::Result<bool> 
    where 
        std::ffi::OsString: From<T>
    {
        Subsystem { nqn: std::ffi::OsString::from(nqn) }.path().try_exists()
    }

    /// Remove the subsystem with the given nqn. This will return an error if a subsystem with the given
    /// nqn does not exist.
    pub fn delete<T>(nqn: T) -> std::io::Result<()>
    where 
        std::ffi::OsString: From<T>
    {
        std::fs::remove_dir(Subsystem{ nqn: std::ffi::OsString::from(nqn) }.path())
    }

    /// Return the host nqn.
    pub fn nqn<'a>(&'a self) -> &'a str {
        &self.nqn.to_str().unwrap()
    }

    /// Create a namespace in the given subsystem with the given nsid. Return an error
    /// if a namespace with the given nsid already exists in the subsystem.
    pub fn create_namespace(&self, nsid: u32) -> std::io::Result<Namespace> {
        let path = self.path().join("namespaces").join(nsid.to_string());
        std::fs::create_dir(&path)?;
        Ok(Namespace { path: path })
    }

    pub fn path(&self) -> std::path::PathBuf {
        std::path::Path::new(CONFIGFS_DIR).join("subsystems").join(&self.nqn)
    }
    
    pub fn set_attr_allow_any_host(&mut self, value: bool) -> std::io::Result<()> {
        let attr_path = self.path().join("attr_allow_any_host");
        let mut file = std::fs::File::create(attr_path)?;
        let value_bytes: &[u8; 2] = match value {
            true => b"1\n",
            false => b"0\n",
        };
        file.write_all(value_bytes)?;
        Ok(())
    }

    pub fn attr_allow_any_host(&self) -> std::io::Result<bool> {
        let attr_allow_any_host_path = self.path().join("attr_allow_any_host");
        let attr_allow_any_host_str = std::fs::read_to_string(attr_allow_any_host_path).unwrap();
        let attr_allow_any_host = attr_allow_any_host_str == "1\n";
        Ok(attr_allow_any_host)
    }

    pub fn set_attr_cntlid_max(&mut self, value: u16) -> std::io::Result<()> {
        let attr_path = self.path().join("attr_cntlid_max");
        let mut file = std::fs::File::create(attr_path)?;
        let value_string =  value.to_string() + "\n";
        file.write_all(value_string.as_bytes())?;
        Ok(())
    }

    pub fn attr_cntlid_max(&self) -> std::io::Result<u16> {
        let attr_cntlid_max_path = self.path().join("attr_cntlid_max");
        let attr_cntlid_max_str = std::fs::read_to_string(attr_cntlid_max_path).unwrap();
        let attr_cntlid_max = attr_cntlid_max_str.trim_end_matches('\n').parse::<u16>().unwrap();
        Ok(attr_cntlid_max)
    }

    pub fn set_attr_cntlid_min(&mut self, value: u16) -> std::io::Result<()> {
        let attr_path = self.path().join("attr_cntlid_min");
        let mut file = std::fs::File::create(attr_path)?;
        let value_string =  value.to_string() + "\n";
        file.write_all(value_string.as_bytes())?;
        Ok(())
    }

    pub fn attr_cntlid_min(&self) -> std::io::Result<u16> {
        let attr_cntlid_min_path = self.path().join("attr_cntlid_min");
        let attr_cntlid_min_str = std::fs::read_to_string(attr_cntlid_min_path).unwrap();
        let attr_cntlid_min = attr_cntlid_min_str.trim_end_matches('\n').parse::<u16>().unwrap();
        Ok(attr_cntlid_min)
    }

    pub fn set_attr_model(&mut self, value: &str) -> std::io::Result<()> {
        let attr_path = self.path().join("attr_model");
        let mut file = std::fs::File::create(attr_path)?;
        let value_string = value.to_string() + "\n";
        file.write_all(value_string.as_bytes())?;
        Ok(())
    }

    pub fn attr_model(&self) -> std::io::Result<String> {
        let attr_model_path = self.path().join("attr_model");
        let attr_model_str = std::fs::read_to_string(attr_model_path).unwrap();
        Ok(attr_model_str.trim_end_matches('\n').to_string())
    }

    pub fn set_attr_serial(&mut self, value: &str) -> std::io::Result<()> {
        let attr_path = self.path().join("attr_serial");
        let mut file = std::fs::File::create(attr_path)?;
        let value_string = value.to_string() + "\n";
        file.write_all(value_string.as_bytes())?;
        Ok(())
    }

    pub fn attr_serial(&self) -> std::io::Result<String> {
        let attr_serial_path = self.path().join("attr_serial");
        let attr_serial_str = std::fs::read_to_string(attr_serial_path).unwrap();
        Ok(attr_serial_str.trim_end_matches('\n').to_string())
    }

    pub fn namespaces(&self) -> std::io::Result<impl Iterator<Item = Namespace> + '_> {
        let namespace_dir = self.path().join("namespaces");
        let namespace_paths = std::fs::read_dir(namespace_dir)?;
        Ok(namespace_paths.map(|namespace_path| {
            let namespace_path = namespace_path.unwrap();
            Namespace{
                path: namespace_path.path()
            }
        }))
    }

    pub fn list_all() -> std::io::Result<impl Iterator<Item = Subsystem>> {
        let path = std::path::Path::new(CONFIGFS_DIR).join("subsystems");
        let paths = std::fs::read_dir(path)?;
        Ok(paths.map(|path| {
            Subsystem { nqn: path.unwrap().path().file_name().unwrap().to_os_string() }
        }))
    }
}

pub struct Port {
    id: u32
}

impl Port {
    /// Add a new Port with the given id. This will return an error if a Port with the
    /// given id already exists.
    pub fn new(id: u32) -> std::io::Result<Port> {
        let port = Port { id };
        std::fs::DirBuilder::new().recursive(true).create(&port.path())?;
        Ok(port)
    }

    /// Return the Host with the given id. This will not return an error if the host does
    /// not exist.
    pub fn open(id: u32) -> Self {
        Self { id }
    }

    /// Return a boolean indicating whether or not a Port with the given id exists.
    pub fn exists(id: u32) -> std::io::Result<bool> {
        Port { id }.path().try_exists()
    }

    /// Remove the Port with the given id. This will return an error if a Port with the given
    /// id does not exist.
    pub fn delete(id: u32) -> std::io::Result<()> {
        std::fs::remove_dir(Port{ id }.path())
    }

    /// Return the Port id.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Return the Port configfs path.
    pub fn path(&self) -> std::path::PathBuf {
        std::path::Path::new(CONFIGFS_DIR).join("ports").join(self.id.to_string())
    }

    pub fn subsystems(&self) -> std::io::Result<impl Iterator<Item = Subsystem>> {
        let path = self.path().join("subsystems");
        let subsystems = std::fs::read_dir(path)?;
        Ok(subsystems.map(|subsys_path| Subsystem {
            nqn: subsys_path.unwrap().path().file_name().unwrap().to_os_string()
        }))
    }

    pub fn has_subsystem(&self, subsys: &Subsystem) -> std::io::Result<bool> {
        let res = std::fs::read_link(
            self.path().join("subsystems").join(&subsys.nqn())
        );
        match res {
            Ok(_) => Ok(true),
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return Ok(false)
                }
                Err(err)
            }
        }
    }

    pub fn add_subsystem(&self, subsys: &Subsystem) -> std::io::Result<()> {
        std::os::unix::fs::symlink(
            subsys.path(), 
            self.path().join("subsystems").join(&subsys.nqn())
        )
    }

    pub fn remove_subsystem(&self, nqn: &str) -> std::io::Result<()> {
        std::fs::remove_file(self.path().join("subsystems").join(nqn))
    }

    pub fn set_addr_adrfam(&mut self, value: &str) -> std::io::Result<()> {
        let path = self.path().join("addr_adrfam");
        let mut file = std::fs::File::create(path)?;
        let value_string = value.to_string() + "\n";
        file.write_all(value_string.as_bytes())?;
        Ok(())
    }

    pub fn addr_adrfam(&self)-> std::result::Result<String, ReadError<String>> {
        read(self.path().join("addr_adrfam"))
    }

    pub fn set_addr_traddr(&mut self, value: &str) -> std::io::Result<()> {
        let path = self.path().join("addr_traddr");
        let mut file = std::fs::File::create(path)?;
        let value_string = value.to_string() + "\n";
        file.write_all(value_string.as_bytes())?;
        Ok(())
    }

    pub fn addr_traddr(&self)-> std::result::Result<String, ReadError<String>> {
        read(self.path().join("addr_traddr"))
    }

    pub fn set_addr_trsvcid(&mut self, value: &str) -> std::io::Result<()> {
        let path = self.path().join("addr_trsvcid");
        let mut file = std::fs::File::create(path)?;
        let value_string = value.to_string() + "\n";
        file.write_all(value_string.as_bytes())?;
        Ok(())
    }

    pub fn addr_trsvcid(&self)-> std::result::Result<String, ReadError<String>> {
        read(self.path().join("addr_trsvcid"))
    }

    pub fn set_addr_trtype(&mut self, value: &str) -> std::io::Result<()> {
        let path = self.path().join("addr_trtype");
        let mut file = std::fs::File::create(path)?;
        let value_string = value.to_string() + "\n";
        file.write_all(value_string.as_bytes())?;
        Ok(())
    }

    pub fn addr_trtype(&self)-> std::result::Result<String, ReadError<String>> {
        read(self.path().join("addr_trtype"))
    }
}

pub struct Host {
    nqn: std::ffi::OsString
}

impl Host {
    /// Add a new Host with the given nqn. This will fail if a host with the
    /// given nqn already exists.
    pub fn new<T>(nqn: T) -> std::io::Result<Self> 
    where 
        std::ffi::OsString: From<T>
    {
        let host = Self { nqn: std::ffi::OsString::from(nqn) };
        std::fs::create_dir(&host.path())?;
        Ok(host)
    }

    /// Return a boolean indicating whether or not a Host with the given nqn exists.
    pub fn exists<T>(nqn: T) -> std::io::Result<bool>
    where 
        std::ffi::OsString: From<T>
    {
        Self { nqn: std::ffi::OsString::from(nqn) }.path().try_exists()
    }

    /// Remove the Host with the given nqn. This will return an error if a host
    /// with the given nqn does not exist.
    pub fn delete<T>(nqn: T) -> std::io::Result<()>
    where 
        std::ffi::OsString: From<T>
    {
        let host = Self { nqn: std::ffi::OsString::from(nqn) };
        std::fs::remove_dir(host.path())
    }

    /// Return the Host configfs path.
    pub fn path(&self) -> std::path::PathBuf {
        std::path::Path::new(CONFIGFS_DIR).join("hosts").join(self.nqn.clone())
    }

    /// Return the Host nqn.
    pub fn nqn<'a>(&'a self) -> &'a str {
        &self.nqn.to_str().unwrap()
    }
}
