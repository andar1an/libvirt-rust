/*
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library.  If not, see
 * <https://www.gnu.org/licenses/>.
 *
 * Sahid Orentino Ferdjaoui <sahid.ferdjaoui@redhat.com>
 */

use crate::connect::Connect;
use crate::error::Error;
use crate::util::{check_neg, check_null};

/// Provides APIs for the management of interfaces.
///
/// See <https://libvirt.org/html/libvirt-libvirt-interface.html>
#[derive(Debug)]
pub struct Interface {
    ptr: sys::virInterfacePtr,
}

unsafe impl Send for Interface {}
unsafe impl Sync for Interface {}

impl Drop for Interface {
    fn drop(&mut self) {
        if let Err(e) = check_neg!(unsafe { sys::virInterfaceFree(self.as_ptr()) }) {
            panic!("Unable to drop reference on interface: {e}")
        }
    }
}

impl Clone for Interface {
    /// Creates a copy of a interface.
    ///
    /// Increments the internal reference counter on the given
    /// interface.
    fn clone(&self) -> Self {
        if let Err(e) = check_neg!(unsafe { sys::virInterfaceRef(self.as_ptr()) }) {
            panic!("Unable to add reference on interface: {e}")
        }
        unsafe { Interface::from_ptr(self.as_ptr()) }
    }
}

impl Interface {
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid.
    /// The rust wrapper will own the reference count
    /// for the C object upon return.
    pub unsafe fn from_ptr(ptr: sys::virInterfacePtr) -> Interface {
        Interface { ptr }
    }

    /// # Safety
    ///
    /// The pointer returned by this method is a copy of
    /// a pointer that is normally tracked by reference
    /// counting in the underlying implementation. Creating
    /// a copy of the pointer explicitly circumvents that
    /// reference counting. The returned pointer may be
    /// invalidated if this object is dropped.
    pub unsafe fn as_ptr(&self) -> sys::virInterfacePtr {
        self.ptr
    }

    pub fn connect(&self) -> Result<Connect, Error> {
        let ptr = check_null!(unsafe { sys::virInterfaceGetConnect(self.as_ptr()) })?;
        if let Err(e) = check_neg!(unsafe { sys::virConnectRef(ptr) }) {
            panic!("Unable to add reference on connection: {e}")
        }
        Ok(unsafe { Connect::from_ptr(ptr) })
    }

    /// Returns the interface name
    ///
    /// See <https://libvirt.org/html/libvirt-libvirt-interface.html#virInterfaceGetName>
    pub fn name(&self) -> Result<String, Error> {
        let n = check_null!(unsafe { sys::virInterfaceGetName(self.as_ptr()) })?;
        Ok(unsafe { c_chars_to_string!(n, nofree) })
    }

    /// Returns the interface MAC address string
    ///
    /// See <https://libvirt.org/html/libvirt-libvirt-interface.html#virInterfaceGetMACString>
    pub fn mac_string(&self) -> Result<String, Error> {
        let mac = check_null!(unsafe { sys::virInterfaceGetMACString(self.as_ptr()) })?;
        Ok(unsafe { c_chars_to_string!(mac, nofree) })
    }

    /// Returns the interface XML configuration
    ///
    /// See <https://libvirt.org/html/libvirt-libvirt-interface.html#virInterfaceGetXMLDesc>
    pub fn xml_desc(&self, flags: sys::virInterfaceXMLFlags) -> Result<String, Error> {
        let xml = check_null!(unsafe { sys::virInterfaceGetXMLDesc(self.as_ptr(), flags) })?;
        Ok(unsafe { c_chars_to_string!(xml) })
    }

    /// Starts the inactive interface
    ///
    /// See <https://libvirt.org/html/libvirt-libvirt-interface.html#virInterfaceCreate>
    pub fn create(&self, flags: sys::virInterfaceXMLFlags) -> Result<(), Error> {
        let _ = check_neg!(unsafe { sys::virInterfaceCreate(self.as_ptr(), flags) })?;
        Ok(())
    }

    /// Stops the active interface
    ///
    /// See <https://libvirt.org/html/libvirt-libvirt-interface.html#virInterfaceDestroy>
    pub fn destroy(&self, flags: u32) -> Result<(), Error> {
        let _ = check_neg!(unsafe { sys::virInterfaceDestroy(self.as_ptr(), flags) })?;
        Ok(())
    }

    /// Removes the interface configuration
    ///
    /// See <https://libvirt.org/html/libvirt-libvirt-interface.html#virInterfaceUndefine>
    pub fn undefine(&self) -> Result<(), Error> {
        let _ = check_neg!(unsafe { sys::virInterfaceUndefine(self.as_ptr()) })?;
        Ok(())
    }

    /// Determines if the interface is active
    ///
    /// See <https://libvirt.org/html/libvirt-libvirt-interface.html#virInterfaceIsActive>
    pub fn is_active(&self) -> Result<bool, Error> {
        let ret = check_neg!(unsafe { sys::virInterfaceIsActive(self.as_ptr()) })?;
        Ok(ret == 1)
    }
}
