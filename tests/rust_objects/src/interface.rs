/* generated by rust_qt_binding_generator */
#![allow(unknown_lints)]
#![allow(mutex_atomic, needless_pass_by_value)]
use libc::{c_char, c_ushort, c_int};
use std::slice;
use std::char::decode_utf16;

use std::sync::{Arc, Mutex};
use std::ptr::null;

use implementation::*;


pub enum QString {}

fn set_string_from_utf16(s: &mut String, str: *const c_ushort, len: c_int) {
    let utf16 = unsafe { slice::from_raw_parts(str, to_usize(len)) };
    let characters = decode_utf16(utf16.iter().cloned())
        .into_iter()
        .map(|r| r.unwrap());
    s.clear();
    s.extend(characters);
}



fn to_usize(n: c_int) -> usize {
    if n < 0 {
        panic!("Cannot cast {} to usize", n);
    }
    n as usize
}


fn to_c_int(n: usize) -> c_int {
    if n > c_int::max_value() as usize {
        panic!("Cannot cast {} to c_int", n);
    }
    n as c_int
}


pub struct GroupQObject {}

#[derive(Clone)]
pub struct GroupEmitter {
    qobject: Arc<Mutex<*const GroupQObject>>,
}

unsafe impl Send for GroupEmitter {}

impl GroupEmitter {
    fn clear(&self) {
        *self.qobject.lock().unwrap() = null();
    }
}

pub trait GroupTrait {
    fn new(emit: GroupEmitter,
        person: Person) -> Self;
    fn emit(&self) -> &GroupEmitter;
    fn person(&self) -> &Person;
    fn person_mut(&mut self) -> &mut Person;
}

#[no_mangle]
pub extern "C" fn group_new(
    group: *mut GroupQObject,
    person: *mut PersonQObject,
    object: *mut InnerObjectQObject,
    description_changed: fn(*const InnerObjectQObject),
) -> *mut Group {
    let object_emit = InnerObjectEmitter {
        qobject: Arc::new(Mutex::new(object)),
        description_changed: description_changed,
    };
    let d_object = InnerObject::new(object_emit);
    let person_emit = PersonEmitter {
        qobject: Arc::new(Mutex::new(person)),
    };
    let d_person = Person::new(person_emit,
        d_object);
    let group_emit = GroupEmitter {
        qobject: Arc::new(Mutex::new(group)),
    };
    let d_group = Group::new(group_emit,
        d_person);
    Box::into_raw(Box::new(d_group))
}

#[no_mangle]
pub unsafe extern "C" fn group_free(ptr: *mut Group) {
    Box::from_raw(ptr).emit().clear();
}

#[no_mangle]
pub unsafe extern "C" fn group_person_get(ptr: *mut Group) -> *mut Person {
    (&mut *ptr).person_mut()
}

pub struct InnerObjectQObject {}

#[derive(Clone)]
pub struct InnerObjectEmitter {
    qobject: Arc<Mutex<*const InnerObjectQObject>>,
    description_changed: fn(*const InnerObjectQObject),
}

unsafe impl Send for InnerObjectEmitter {}

impl InnerObjectEmitter {
    fn clear(&self) {
        *self.qobject.lock().unwrap() = null();
    }
    pub fn description_changed(&self) {
        let ptr = *self.qobject.lock().unwrap();
        if !ptr.is_null() {
            (self.description_changed)(ptr);
        }
    }
}

pub trait InnerObjectTrait {
    fn new(emit: InnerObjectEmitter) -> Self;
    fn emit(&self) -> &InnerObjectEmitter;
    fn description(&self) -> &str;
    fn set_description(&mut self, value: String);
}

#[no_mangle]
pub extern "C" fn inner_object_new(
    inner_object: *mut InnerObjectQObject,
    description_changed: fn(*const InnerObjectQObject),
) -> *mut InnerObject {
    let inner_object_emit = InnerObjectEmitter {
        qobject: Arc::new(Mutex::new(inner_object)),
        description_changed: description_changed,
    };
    let d_inner_object = InnerObject::new(inner_object_emit);
    Box::into_raw(Box::new(d_inner_object))
}

#[no_mangle]
pub unsafe extern "C" fn inner_object_free(ptr: *mut InnerObject) {
    Box::from_raw(ptr).emit().clear();
}

#[no_mangle]
pub extern "C" fn inner_object_description_get(
    ptr: *const InnerObject,
    p: *mut QString,
    set: fn(*mut QString, *const c_char, c_int),
) {
    let o = unsafe { &*ptr };
    let v = o.description();
    let s: *const c_char = v.as_ptr() as (*const c_char);
    set(p, s, to_c_int(v.len()));
}

#[no_mangle]
pub extern "C" fn inner_object_description_set(ptr: *mut InnerObject, v: *const c_ushort, len: c_int) {
    let o = unsafe { &mut *ptr };
    let mut s = String::new();
    set_string_from_utf16(&mut s, v, len);
    o.set_description(s);
}

pub struct PersonQObject {}

#[derive(Clone)]
pub struct PersonEmitter {
    qobject: Arc<Mutex<*const PersonQObject>>,
}

unsafe impl Send for PersonEmitter {}

impl PersonEmitter {
    fn clear(&self) {
        *self.qobject.lock().unwrap() = null();
    }
}

pub trait PersonTrait {
    fn new(emit: PersonEmitter,
        object: InnerObject) -> Self;
    fn emit(&self) -> &PersonEmitter;
    fn object(&self) -> &InnerObject;
    fn object_mut(&mut self) -> &mut InnerObject;
}

#[no_mangle]
pub extern "C" fn person_new(
    person: *mut PersonQObject,
    object: *mut InnerObjectQObject,
    description_changed: fn(*const InnerObjectQObject),
) -> *mut Person {
    let object_emit = InnerObjectEmitter {
        qobject: Arc::new(Mutex::new(object)),
        description_changed: description_changed,
    };
    let d_object = InnerObject::new(object_emit);
    let person_emit = PersonEmitter {
        qobject: Arc::new(Mutex::new(person)),
    };
    let d_person = Person::new(person_emit,
        d_object);
    Box::into_raw(Box::new(d_person))
}

#[no_mangle]
pub unsafe extern "C" fn person_free(ptr: *mut Person) {
    Box::from_raw(ptr).emit().clear();
}

#[no_mangle]
pub unsafe extern "C" fn person_object_get(ptr: *mut Person) -> *mut InnerObject {
    (&mut *ptr).object_mut()
}
