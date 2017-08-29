/* generated by rust_qt_binding_generator */
#![allow(unknown_lints)]
#![allow(mutex_atomic, needless_pass_by_value)]
use libc::{c_int, c_void, uint8_t, uint16_t};
use std::slice;

use std::sync::{Arc, Mutex};
use std::ptr::null;

use implementation::*;


#[repr(C)]
pub struct QString {
    data: *const uint8_t,
    len: c_int,
}

#[repr(C)]
pub struct QStringIn {
    data: *const uint16_t,
    len: c_int,
}

impl QStringIn {
    fn convert(&self) -> String {
        let data = unsafe { slice::from_raw_parts(self.data, self.len as usize) };
        String::from_utf16_lossy(data)
    }
}

impl<'a> From<&'a String> for QString {
    fn from(string: &'a String) -> QString {
        QString {
            len: string.len() as c_int,
            data: string.as_ptr(),
        }
    }
}

pub struct PersonQObject {}

#[derive(Clone)]
pub struct PersonEmitter {
    qobject: Arc<Mutex<*const PersonQObject>>,
    user_name_changed: fn(*const PersonQObject),
}

unsafe impl Send for PersonEmitter {}

impl PersonEmitter {
    fn clear(&self) {
        *self.qobject.lock().unwrap() = null();
    }
    pub fn user_name_changed(&self) {
        let ptr = *self.qobject.lock().unwrap();
        if !ptr.is_null() {
            (self.user_name_changed)(ptr);
        }
    }
}

pub trait PersonTrait {
    fn create(emit: PersonEmitter) -> Self;
    fn emit(&self) -> &PersonEmitter;
    fn get_user_name(&self) -> String;
    fn set_user_name(&mut self, value: String);
}

#[no_mangle]
pub extern "C" fn person_new(
    person: *mut PersonQObject,
    user_name_changed: fn(*const PersonQObject),
) -> *mut Person {
    let person_emit = PersonEmitter {
        qobject: Arc::new(Mutex::new(person)),
        user_name_changed: user_name_changed,
    };
    let d_person = Person::create(person_emit);
    Box::into_raw(Box::new(d_person))
}

#[no_mangle]
pub unsafe extern "C" fn person_free(ptr: *mut Person) {
    Box::from_raw(ptr).emit().clear();
}

#[no_mangle]
pub unsafe extern "C" fn person_user_name_get(
    ptr: *const Person,
    p: *mut c_void,
    set: fn(*mut c_void, QString),
) {
    let data = (&*ptr).get_user_name();
    set(p, QString::from(&data));
}

#[no_mangle]
pub unsafe extern "C" fn person_user_name_set(ptr: *mut Person, v: QStringIn) {
    (&mut *ptr).set_user_name(v.convert());
}
