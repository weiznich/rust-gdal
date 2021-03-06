extern crate gdal;

use std::path::Path;
use std::fs;
use gdal::vector::{Defn, Driver, Feature, FieldDefn, Geometry, OGRFieldType, FieldValue};

fn main(){
    /// Example 1, the detailed way :
    {
        fs::remove_file("/tmp/output1.geojson");
        let drv = Driver::get("GeoJSON").unwrap();
        let mut ds = drv.create(Path::new("/tmp/output1.geojson")).unwrap();

        let lyr = ds.create_layer().unwrap();

        let field_defn = FieldDefn::new("Name", OGRFieldType::OFTString).unwrap();
        field_defn.set_width(80);
        field_defn.add_to_layer(&lyr);

        let field_defn = FieldDefn::new("Value", OGRFieldType::OFTReal).unwrap();
        field_defn.add_to_layer(&lyr);

        let defn = Defn::from_layer(&lyr);

        // 1st feature :
        let mut ft = Feature::new(&defn).unwrap();
        ft.set_geometry(Geometry::from_wkt("POINT (45.21 21.76)").unwrap());
        ft.set_field_string("Name", "Feature 1");
        ft.set_field_double("Value", 45.78);
        ft.create(&lyr);

        // 2nd feature :
        let mut ft = Feature::new(&defn).unwrap();
        ft.set_geometry(Geometry::from_wkt("POINT (46.50 22.50)").unwrap());
        ft.set_field_string("Name", "Feature 2");
        ft.set_field_double("Value", 0.789);
        ft.create(&lyr);

        // Feature triggering an error due to a wrong field name :
        let mut ft = Feature::new(&defn).unwrap();
        ft.set_geometry(Geometry::from_wkt("POINT (46.50 22.50)").unwrap());
        ft.set_field_string("Name", "Feature 2");
        match ft.set_field_double("Values", 0.789) {
            Ok(v) => v, Err(err) => println!("{:?}", err.to_string()),
        };
        ft.create(&lyr);
    }

    /// Example 2, same output, shortened way :
    {
        fs::remove_file("/tmp/output2.geojson");
        let driver = Driver::get("GeoJSON").unwrap();
        let mut ds = driver.create(Path::new("/tmp/output2.geojson")).unwrap();
        let mut layer = ds.create_layer().unwrap();

        layer.create_defn_fields(&[("Name", OGRFieldType::OFTString), ("Value", OGRFieldType::OFTReal)]);
        // Shortcut for :
        // let field_defn = FieldDefn::new("Name", OFT_STRING);
        // field_defn.add_to_layer(&layer);
        // let field_defn = FieldDefn::new("Value", OFT_REAL);
        // field_defn.add_to_layer(&layer);

        layer.create_feature_fields(
            Geometry::from_wkt("POINT (45.21 21.76)").unwrap(),
            &["Name", "Value"],
            &[FieldValue::StringValue("Feature 1".to_string()), FieldValue::RealValue(45.78)]
        ).unwrap();

        layer.create_feature_fields(
            Geometry::from_wkt("POINT (46.50 22.50)").unwrap(),
            &["Name", "Value"],
            &[FieldValue::StringValue("Feature 2".to_string()), FieldValue::RealValue(0.789)]
        ).unwrap();

        // Feature creation triggering an error due to a wrong field name :
        match layer.create_feature_fields(
                Geometry::from_wkt("POINT (46.50 22.50)").unwrap(),
                &["Abcd", "Value"],
                &[FieldValue::StringValue("Feature 2".to_string()), FieldValue::RealValue(0.789)]) {
            Ok(v) => v,
            Err(err) => println!("{:?}", err.to_string()),
        };
        // Shortcuts for :
        // let defn = Defn::new_from_layer(&layer);
        //
        // let mut ft = Feature::new(&defn);
        // ft.set_geometry(Geometry::from_wkt("POINT (45.21 21.76)").unwrap());
        // ft.set_field("Name", OFT_STRING, "Feature 1");
        // ft.set_field("Value", OFT_REAL, 45.78);
        // ft.create(&lyr);
        //
        // let mut ft = Feature::new(&defn);
        // ft.set_geometry(Geometry::from_wkt("POINT (46.50 22.50)").unwrap());
        // ft.set_field("Name", OFT_STRING, "Feature 2");
        // ft.set_field("Value", OFT_REAL, 0.789);
        // ft.create(&lyr);
    }

}
