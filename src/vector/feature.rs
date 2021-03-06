use std::ffi::CString;
use libc::{c_void, c_double, c_int};
use vector::Defn;
use utils::{_string, _last_null_pointer_err};
use gdal_sys::{ogr, ogr_enums};
use vector::geometry::Geometry;
use vector::layer::Layer;
use gdal_major_object::MajorObject;
use gdal_sys::ogr_enums::OGRFieldType;

use errors::*;

/// OGR Feature
pub struct Feature<'a> {
    _defn: &'a Defn,
    c_feature: *const c_void,
    geometry: Vec<Geometry>,
}


impl<'a> Feature<'a> {
    pub fn new(defn: &'a Defn) -> Result<Feature> {
        let c_feature = unsafe { ogr::OGR_F_Create(defn.c_defn()) };
        if c_feature.is_null() {
            return Err(_last_null_pointer_err("OGR_F_Create").into());
        };
        Ok(Feature {
                 _defn: defn,
                 c_feature: c_feature,
                 geometry: Feature::_lazy_feature_geometries(defn),
             })
    }

    pub unsafe fn _with_c_feature(defn: &'a Defn, c_feature: *const c_void) -> Feature {
        return Feature{
            _defn: defn,
            c_feature: c_feature,
            geometry: Feature::_lazy_feature_geometries(defn),
        };
    }

    pub fn _lazy_feature_geometries(defn: &'a Defn) -> Vec<Geometry> {
        let geom_field_count = unsafe { ogr::OGR_FD_GetGeomFieldCount(defn.c_defn()) } as isize;
        let geometries = (0..geom_field_count).map(|_| unsafe { Geometry::lazy_feature_geometry() }).collect();
        geometries
    }

    /// Get the value of a named field. If the field exists, it returns a
    /// `FieldValue` wrapper, that you need to unpack to a base type
    /// (string, float, etc). If the field is missing, returns `None`.
    pub fn field(&self, name: &str) -> Result<FieldValue> {
        let c_name = CString::new(name)?;
        let field_id = unsafe { ogr::OGR_F_GetFieldIndex(self.c_feature, c_name.as_ptr()) };
        if field_id == -1 {
            return Err(ErrorKind::InvalidFieldName(name.to_string(), "field").into());
        }
        let field_defn = unsafe { ogr::OGR_F_GetFieldDefnRef(self.c_feature, field_id) };
        let field_type = unsafe { ogr::OGR_Fld_GetType(field_defn) };
        match field_type {
            OGRFieldType::OFTString => {
                let rv = unsafe { ogr::OGR_F_GetFieldAsString(self.c_feature, field_id) };
                return Ok(FieldValue::StringValue(_string(rv)));
            },
            OGRFieldType::OFTReal => {
                let rv = unsafe { ogr::OGR_F_GetFieldAsDouble(self.c_feature, field_id) };
                return Ok(FieldValue::RealValue(rv as f64));
            },
            OGRFieldType::OFTInteger => {
                let rv = unsafe { ogr::OGR_F_GetFieldAsInteger(self.c_feature, field_id) };
                return Ok(FieldValue::IntegerValue(rv as i32));
            },
            _ => Err(ErrorKind::UnhandledFieldType(field_type, "OGR_Fld_GetType").into())
        }
    }

    /// Get the field's geometry.
    pub fn geometry(&self) -> &Geometry {
        if ! self.geometry[0].has_gdal_ptr() {
            let c_geom = unsafe { ogr::OGR_F_GetGeometryRef(self.c_feature) };
            unsafe { self.geometry[0].set_c_geometry(c_geom) };
        }
        return &self.geometry[0];
    }

    pub fn geometry_by_name(&self, field_name: &str) -> Result<&Geometry> {
        let c_str_field_name = CString::new(field_name)?;
        let idx = unsafe { ogr::OGR_F_GetGeomFieldIndex(self.c_feature, c_str_field_name.as_ptr())};
        if idx == -1 {
            Err(ErrorKind::InvalidFieldName(field_name.to_string(), "geometry_by_name").into())
        } else {
            self.geometry_by_index(idx as usize)
        }
    }

    pub fn geometry_by_index(&self, idx: usize) -> Result<&Geometry> {
        if idx >= self.geometry.len() {
            return Err(ErrorKind::InvalidFieldIndex(idx, "geometry_by_name").into())
        }
        if ! self.geometry[idx].has_gdal_ptr() {
            let c_geom = unsafe { ogr::OGR_F_GetGeomFieldRef(self.c_feature, idx as i32) };
            if c_geom.is_null() {
                return Err(_last_null_pointer_err("OGR_F_GetGeomFieldRef").into());
            }
            unsafe { self.geometry[idx].set_c_geometry(c_geom) };
        }
        Ok(&self.geometry[idx])
    }

    pub fn create(&self, lyr: &Layer) -> Result<()> {
        let rv = unsafe { ogr::OGR_L_CreateFeature(lyr.gdal_object_ptr(), self.c_feature) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_L_CreateFeature").into());
        }
        Ok(())
    }

    pub fn set_field_string(&self, field_name: &str, value: &str) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let c_str_value = CString::new(value)?;
        let idx = unsafe { ogr::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr())};
        if idx == -1 {
            return Err(ErrorKind::InvalidFieldName(field_name.to_string(), "set_field_string").into());
        }
        unsafe { ogr::OGR_F_SetFieldString(self.c_feature, idx, c_str_value.as_ptr()) };
        Ok(())
    }

    pub fn set_field_double(&self, field_name: &str, value: f64) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let idx = unsafe { ogr::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr())};
        if idx == -1 {
            return Err(ErrorKind::InvalidFieldName(field_name.to_string(), "set_field_string").into());
        }
        unsafe { ogr::OGR_F_SetFieldDouble(self.c_feature, idx, value as c_double) };
        Ok(())
    }

    pub fn set_field_integer(&self, field_name: &str, value: i32) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let idx = unsafe { ogr::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr())};
        if idx == -1 {
            return Err(ErrorKind::InvalidFieldName(field_name.to_string(), "set_field_string").into());
        }
        unsafe { ogr::OGR_F_SetFieldInteger(self.c_feature, idx, value as c_int) };
        Ok(())
    }

    pub fn set_field(&self, field_name: &str,  value: &FieldValue) -> Result<()> {
          match value {
             &FieldValue::RealValue(value) => self.set_field_double(field_name, value),
             &FieldValue::StringValue(ref value) => self.set_field_string(field_name, value.as_str()),
             &FieldValue::IntegerValue(value) => self.set_field_integer(field_name, value)
         }
     }

    pub fn set_geometry(&mut self, geom: Geometry) -> Result<()> {
        let rv = unsafe { ogr::OGR_F_SetGeometry(self.c_feature, geom.c_geometry()) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_G_SetGeometry").into());
        }
        self.geometry[0] = geom;
        Ok(())
    }
}


impl<'a> Drop for Feature<'a> {
    fn drop(&mut self) {
        unsafe { ogr::OGR_F_Destroy(self.c_feature); }
    }
}


pub enum FieldValue {
    IntegerValue(i32),
    StringValue(String),
    RealValue(f64),
}


impl FieldValue {
    /// Interpret the value as `String`. Panics if the value is something else.
    pub fn to_string(self) -> Option<String> {
        match self {
            FieldValue::StringValue(rv) => Some(rv),
            _ => None
        }
    }

    /// Interpret the value as `f64`. Panics if the value is something else.
    pub fn to_real(self) -> Option<f64> {
        match self {
            FieldValue::RealValue(rv) => Some(rv),
            _ => None
        }
    }

    /// Interpret the value as `i32`. Panics if the value is something else.
    pub fn to_int(self) -> Option<i32> {
        match self {
            FieldValue::IntegerValue(rv) => Some(rv),
            _ => None
        }
    }
}
