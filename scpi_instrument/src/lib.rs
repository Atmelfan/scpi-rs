#![no_std]

use scpi::command::Command;
use scpi::error::Error;

trait Abort {
    fn abort(&self) -> Result<(), Error>;
}

trait Configure {
    fn configure(&self) -> Result<(), Error>;
}

trait ReadInit {
    fn initiate(&self) -> Result<(), Error>;
}

trait ReadFetch {
    fn fetch(&self) -> Result<(), Error>;
}

trait Read: Abort + ReadInit + ReadFetch {
    fn read(&self) -> Result<(), Error> {
        self.abort()?;
        self.initiate()?;
        self.fetch()
    }
}
