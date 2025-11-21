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

use std::ffi::CString;
use std::{mem, str};

use crate::connect::Connect;
use crate::error::Error;
use crate::storage_pool::StoragePool;
use crate::stream::Stream;
use crate::util::{check_neg, check_null};

#[derive(Clone, Debug)]
pub struct StorageVolInfo {
    /// See: `virStorageVolType` flags
    pub kind: u32,
    /// Logical size bytes.
    pub capacity: u64,
    /// Current allocation bytes
    pub allocation: u64,
}

impl StorageVolInfo {
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid.
    /// The rust wrapper will own the reference count
    /// for the C object upon return.
    pub unsafe fn from_ptr(ptr: sys::virStorageVolInfoPtr) -> StorageVolInfo {
        StorageVolInfo {
            kind: (*ptr).type_ as sys::virStorageVolType,
            capacity: (*ptr).capacity,
            allocation: (*ptr).allocation,
        }
    }
}

/// Provides APIs for the management of storage volumes.
///
/// See <https://libvirt.org/html/libvirt-libvirt-storage.html>
#[derive(Debug)]
pub struct StorageVol {
    ptr: sys::virStorageVolPtr,
}

unsafe impl Send for StorageVol {}
unsafe impl Sync for StorageVol {}

impl Drop for StorageVol {
    fn drop(&mut self) {
        if let Err(e) = check_neg!(unsafe { sys::virStorageVolFree(self.as_ptr()) }) {
            panic!("Unable to drop reference on storage volume: {e}")
        }
    }
}

impl Clone for StorageVol {
    /// Creates a copy of a storage pool.
    ///
    /// Increments the internal reference counter on the given
    /// volume.
    fn clone(&self) -> Self {
        if let Err(e) = check_neg!(unsafe { sys::virStorageVolRef(self.as_ptr()) }) {
            panic!("Unable to add reference on storage volume: {e}")
        }
        unsafe { StorageVol::from_ptr(self.as_ptr()) }
    }
}

impl StorageVol {
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid.
    pub unsafe fn from_ptr(ptr: sys::virStorageVolPtr) -> StorageVol {
        StorageVol { ptr }
    }

    /// # Safety
    ///
    /// The pointer returned by this method is a copy of
    /// a pointer that is normally tracked by reference
    /// counting in the underlying implementation. Creating
    /// a copy of the pointer explicitly circumvents that
    /// reference counting. The returned pointer may be
    /// invalidated if this object is dropped.
    pub unsafe fn as_ptr(&self) -> sys::virStorageVolPtr {
        self.ptr
    }

    pub fn connect(&self) -> Result<Connect, Error> {
        let ptr = check_null!(unsafe { sys::virStorageVolGetConnect(self.as_ptr()) })?;
        if let Err(e) = check_neg!(unsafe { sys::virConnectRef(ptr) }) {
            panic!("Unable to add reference on connection: {e}")
        }
        Ok(unsafe { Connect::from_ptr(ptr) })
    }

    pub fn create_xml(
        pool: &StoragePool,
        xml: &str,
        flags: sys::virStorageVolCreateFlags,
    ) -> Result<StorageVol, Error> {
        let xml_buf = CString::new(xml)?;
        let ptr = check_null!(unsafe {
            sys::virStorageVolCreateXML(pool.as_ptr(), xml_buf.as_ptr(), flags as libc::c_uint)
        })?;
        Ok(unsafe { StorageVol::from_ptr(ptr) })
    }

    pub fn create_xml_from(
        pool: &StoragePool,
        xml: &str,
        vol: &StorageVol,
        flags: sys::virStorageVolCreateFlags,
    ) -> Result<StorageVol, Error> {
        let xml_buf = CString::new(xml)?;
        let ptr = check_null!(unsafe {
            sys::virStorageVolCreateXMLFrom(
                pool.as_ptr(),
                xml_buf.as_ptr(),
                vol.as_ptr(),
                flags as libc::c_uint,
            )
        })?;
        Ok(unsafe { StorageVol::from_ptr(ptr) })
    }

    pub fn lookup_storage_pool(&self) -> Result<StoragePool, Error> {
        let ptr = check_null!(unsafe { sys::virStoragePoolLookupByVolume(self.as_ptr()) })?;
        Ok(unsafe { StoragePool::from_ptr(ptr) })
    }

    pub fn name(&self) -> Result<String, Error> {
        let n = check_null!(unsafe { sys::virStorageVolGetName(self.as_ptr()) })?;
        Ok(unsafe { c_chars_to_string!(n, nofree) })
    }

    pub fn key(&self) -> Result<String, Error> {
        let n = check_null!(unsafe { sys::virStorageVolGetKey(self.as_ptr()) })?;
        Ok(unsafe { c_chars_to_string!(n, nofree) })
    }

    pub fn path(&self) -> Result<String, Error> {
        let n = check_null!(unsafe { sys::virStorageVolGetPath(self.as_ptr()) })?;
        Ok(unsafe { c_chars_to_string!(n) })
    }

    pub fn xml_desc(&self, flags: u32) -> Result<String, Error> {
        let xml = check_null!(unsafe { sys::virStorageVolGetXMLDesc(self.as_ptr(), flags) })?;
        Ok(unsafe { c_chars_to_string!(xml) })
    }

    pub fn delete(&self, flags: u32) -> Result<(), Error> {
        let _ =
            check_neg!(unsafe { sys::virStorageVolDelete(self.as_ptr(), flags as libc::c_uint) })?;
        Ok(())
    }

    pub fn wipe(&self, flags: u32) -> Result<(), Error> {
        let _ =
            check_neg!(unsafe { sys::virStorageVolWipe(self.as_ptr(), flags as libc::c_uint) })?;
        Ok(())
    }

    pub fn wipe_pattern(
        &self,
        algo: sys::virStorageVolWipeAlgorithm,
        flags: u32,
    ) -> Result<(), Error> {
        let _ = check_neg!(unsafe {
            sys::virStorageVolWipePattern(
                self.as_ptr(),
                algo as libc::c_uint,
                flags as libc::c_uint,
            )
        })?;
        Ok(())
    }

    pub fn resize(&self, capacity: u64, flags: u32) -> Result<(), Error> {
        let _ = check_neg!(unsafe {
            sys::virStorageVolResize(
                self.as_ptr(),
                capacity as libc::c_ulonglong,
                flags as libc::c_uint,
            )
        })?;
        Ok(())
    }

    pub fn info(&self) -> Result<StorageVolInfo, Error> {
        let mut pinfo = mem::MaybeUninit::uninit();
        let _ =
            check_neg!(unsafe { sys::virStorageVolGetInfo(self.as_ptr(), pinfo.as_mut_ptr()) })?;
        Ok(unsafe { StorageVolInfo::from_ptr(&mut pinfo.assume_init()) })
    }

    pub fn info_flags(&self, flags: u32) -> Result<StorageVolInfo, Error> {
        let mut pinfo = mem::MaybeUninit::uninit();
        let _ = check_neg!(unsafe {
            sys::virStorageVolGetInfoFlags(self.as_ptr(), pinfo.as_mut_ptr(), flags as libc::c_uint)
        })?;
        Ok(unsafe { StorageVolInfo::from_ptr(&mut pinfo.assume_init()) })
    }

    pub fn download(
        &self,
        stream: &Stream,
        offset: u64,
        length: u64,
        flags: u32,
    ) -> Result<(), Error> {
        let _ = check_neg!(unsafe {
            sys::virStorageVolDownload(
                self.as_ptr(),
                stream.as_ptr(),
                offset as libc::c_ulonglong,
                length as libc::c_ulonglong,
                flags as libc::c_uint,
            )
        })?;
        Ok(())
    }

    pub fn upload(
        &self,
        stream: &Stream,
        offset: u64,
        length: u64,
        flags: u32,
    ) -> Result<(), Error> {
        let _ = check_neg!(unsafe {
            sys::virStorageVolUpload(
                self.as_ptr(),
                stream.as_ptr(),
                offset as libc::c_ulonglong,
                length as libc::c_ulonglong,
                flags as libc::c_uint,
            )
        })?;
        Ok(())
    }
}
