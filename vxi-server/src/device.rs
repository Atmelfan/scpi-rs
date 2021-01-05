
pub enum RemoteLocalControl {
    /// De-assert REN
    DisableRemote,
    /// Assert REN
    EnableRemote,
    /// De-assert REN and send GTL
    DisableRemoteGotoLocal,
    /// Assert REN and address device
    EnableRemoteGotoRemote,
    /// Send LLO
    EnableRemoteLockoutLocal,
    /// Assert REN, address and send LLO
    EnableRemoteGotoRemoteLockoutLocal,
    /// Send GTL
    GotoLocal
}


/// A device
///
/// Receives and execute commands
pub trait Device {

    /// Called when a Remote/Local command is received
    fn set_remote_local(&mut self, rlc: RemoteLocalControl) {
        match rlc {
            RemoteLocalControl::DisableRemote | RemoteLocalControl::DisableRemoteGotoLocal => {
                self.remote_enable(false);
                self.local_lockout(false);
                self.remote(false);
            }
            RemoteLocalControl::EnableRemote => {
                self.remote_enable(true);
            }
            RemoteLocalControl::EnableRemoteGotoRemote => {
                self.remote_enable(true);
                self.remote(true);
            }
            RemoteLocalControl::EnableRemoteLockoutLocal => {
                self.remote_enable(true);
                self.local_lockout(true);
            }
            RemoteLocalControl::EnableRemoteGotoRemoteLockoutLocal => {
                self.remote_enable(true);
                self.local_lockout(true);
                self.remote(true);
            }
            RemoteLocalControl::GotoLocal => {
                self.remote(false);
            }
        }
    }

    /// Called to enable remote control
    fn remote_enable(&mut self, enable: bool);
    /// Called to lockout local control
    fn local_lockout(&mut self, enable: bool);
    /// Called to enter remote control
    fn remote(&mut self, enable: bool);

    /// Called when a trigger command is received
    fn trigger(&mut self);

    /// Clear any operation in progress
    fn clear(&mut self);

    ///
    fn lock(&mut self);
    fn unlock(&mut self);
}

enum LockError {
    ExclusivelyLocked,
    SharedLocked,
}

struct DeviceLock;

trait DeviceMutex {
    fn try_exclusive_lock() -> Result<DeviceLock, LockError>;
}