#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub mod Font;

#[macro_use]
extern crate lazy_static;

use fancy_regex::Regex;
use std::{ops::Index, collections::HashMap, path::PathBuf};
use genpdf::fonts::{from_files};

lazy_static! {
  static ref TrackingNumberRegex: Regex = Regex::new(r"(?<=Consignment )\d{14}").unwrap();
  static ref OrderNumberRegex: Regex = Regex::new(r"(?<=\n)\d{9}(?=\n)").unwrap();
  static ref ShoeInfoRegex: Regex = Regex::new(r"(?P<Name>.+)\n(?P<Size>\d{1,2}(\.\d)? US (\w)+) \| (?P<SKU>[A-Z0-9 ]+) \| (?P<Condition>\w+)").unwrap();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler!(startPdf))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn startPdf(path: &str, folder: bool, out_dir:&str,  app: tauri::AppHandle) {
    let app_dir = app.path_resolver().app_dir().expect("failed to get app dir");
    let font_path = app_dir.join("Poppins-Regular.ttf");
    println!("More complex path exists: {}, with full path: {}", font_path.exists(), font_path.as_path().to_str().unwrap());

    print!("Got input {} with folder: {} \t", path, folder);
    if path.eq("World") {
        print!("Rejected input\n");
    }
    print!("Accepted Input\n");

    let s: String = getPDFString(path.to_string()).unwrap();
    let mut split_s: Vec<String> = splitLabelString(s);
    println!("Extracted {} strings", split_s.len());
    let mut s_iter = split_s.iter();
    let orders:Vec<Order> = s_iter.map(|x1| parseTextToSale(x1.to_owned()))
        .filter(|e| e.is_some())
        .map(|x2| x2.unwrap())
        .collect();
    println!("Extracted {} orders", orders.len());

    let app_dir = app.path_resolver().app_dir().expect("failed to get app dir");
    let res = write_PDF(orders, out_dir.to_string(), app_dir);
    if res {
        print!("Exiting startPDF Successfully");
        let mut diag = tauri::api::dialog::MessageDialogBuilder::new("Erfolgreich erstellt", "Die PDF wurde erfolgreich erstellt");
        diag = diag.buttons(tauri::api::dialog::MessageDialogButtons::Ok);
        diag = diag.kind(tauri::api::dialog::MessageDialogKind::Info);
        diag.show(do_nothing)
    }else{
        let mut diag = tauri::api::dialog::MessageDialogBuilder::new("Fehler bei der Erstellung", "Die PDF konnte leider nicht erstellt werden");
        diag = diag.buttons(tauri::api::dialog::MessageDialogButtons::Ok);
        diag = diag.kind(tauri::api::dialog::MessageDialogKind::Error);
        diag.show(do_nothing)
    }
}

fn do_nothing(a:bool) -> () {
    ()
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash)]
enum ShippingServices {
    DPD,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
struct Shoe {
    ShoeName: String,
    Size: String,
    SKU: String,
    Condition: String,
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
struct Order {
    shipper: ShippingServices,
    tracking_number: String,
    shoe: Shoe,
    order_number: String,
}

fn getPDFString(path: String) -> Result<String, pdf_extract::OutputError> {
    pdf_extract::extract_text(path)
}

fn splitLabelString(input: String) -> Vec<String> {
    let texts: Vec<String> = input
        .split("Responsible delivery - CO2 neutral")
        .map(|s| s.to_string())
        .collect();
    println!("Got {} Individual labels", texts.len());
    texts
}

fn parseTextToSale(input: String) -> Option<Order> {
    if input.len() <= 100 {
        println!("Rejected Sale String for too short text");
        return None;
    }
    //println!("Sale: {}", input);
    println!("Beginning regex match");
    let order_match_temp = OrderNumberRegex.captures(&input);
    let order_match =  order_match_temp.expect("Ordernumber could not be Parsed")?;
    println!("Got: {}", order_match.index(0).to_string());
    let tracking_match = TrackingNumberRegex
        .captures(&input)
        .expect("Trackingnumber could not be parsed")?;
    let shoe_info_match = ShoeInfoRegex
        .captures(&input)
        .expect("Could not match shoe information")?;

    Some(Order {
        shipper: ShippingServices::DPD,
        tracking_number: tracking_match.index(0).to_string(),
        shoe: Shoe {
            ShoeName: shoe_info_match.index("Name").to_string(),
            Size: shoe_info_match.index("Size").to_string(),
            SKU: shoe_info_match.index("SKU").to_string(),
            Condition: shoe_info_match.index("Condition").to_string(),
        },
        order_number: order_match.index(0).to_string(),
    })
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
struct CountedShoes{
    Shoe: Shoe,
    Number: u16,
}

fn group_shoes(orders: Vec<Order>) -> Vec<CountedShoes> {
    let mut map : HashMap<Shoe, u16> = HashMap::new();

    
    let shoes = orders.iter().map(|a| a.to_owned().shoe);
    for shoe in shoes{
        *map.entry(shoe).or_insert(0) += 1;
    }
    
    let mut res: Vec<CountedShoes> = Vec::new();
    for (key, value) in map {
        res.push(CountedShoes { Shoe: key, Number: value });
    }
    res
}


fn write_PDF(orders: Vec<Order>, out_path: String, app_dir: PathBuf) -> bool {
    let default: &'static str = "Roboto";

    
    let mut doc = genpdf::Document::new(from_files(app_dir,
                                                   default, None).unwrap());

    doc.set_title("Pickup and Picksheet");
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10 as u16);
    doc.set_page_decorator(decorator);

    doc.push(genpdf::elements::Paragraph::new("Abholung am ____._____.202__"));
    doc.push(genpdf::elements::Paragraph::new(format!("{} Pakete", orders.len())));
    doc.push(genpdf::elements::Break::new(3));

    let mut pickup_table = genpdf::elements::TableLayout::new(vec![1, 1]);
    pickup_table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
    let mut row = pickup_table.row();
    row.push_element(genpdf::elements::Paragraph::new("Order Nummer"));
    row.push_element(genpdf::elements::Paragraph::new("Tracking Nummer"));
    row.push().expect("Invalid table row");
    for ord in orders.clone() {
        let mut row = pickup_table.row();
        row.push_element(genpdf::elements::Paragraph::new(ord.order_number));
        row.push_element(genpdf::elements::Paragraph::new(ord.tracking_number));
        row.push().expect("Invalid table row");
    }
    doc.push(pickup_table);
    doc.push(genpdf::elements::Paragraph::new("Unterschrift Fahrer DPD"));
    doc.push(genpdf::elements::PageBreak::new());
    doc.push(genpdf::elements::Paragraph::new("Pick Sheet"));
    doc.push(genpdf::elements::Break::new(3));

    let mut pick_table = genpdf::elements::TableLayout::new(vec![3, 2, 1, 1, 1]);
    pick_table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
    let mut row = pick_table.row();
    row.push_element(genpdf::elements::Paragraph::new("Name"));
    row.push_element(genpdf::elements::Paragraph::new("SKU"));
    row.push_element(genpdf::elements::Paragraph::new("Size"));
    row.push_element(genpdf::elements::Paragraph::new("Condition"));
    row.push_element(genpdf::elements::Paragraph::new("Number"));
    row.push().expect("Invalid table row");
    for counted in group_shoes(orders) {
        let mut row = pick_table.row();
        row.push_element(genpdf::elements::Paragraph::new(counted.Shoe.ShoeName));
        row.push_element(genpdf::elements::Paragraph::new(counted.Shoe.SKU));
        row.push_element(genpdf::elements::Paragraph::new(counted.Shoe.Size));
        row.push_element(genpdf::elements::Paragraph::new(counted.Shoe.Condition));
        row.push_element(genpdf::elements::Paragraph::new(format!("{}", counted.Number)));
        row.push().expect("Invalid table row");
    }
    doc.push(pick_table);
    println!("Beginning to write pdf to {}", out_path);
    doc.render_to_file(out_path).expect("Failed to write PDF file");
    println!("Printed PDF");
    
    true
}

#[test]
fn test_group_shoes(){
    let shoe1 = Shoe{
        ShoeName: "Test".to_string(),
        Size: "12".to_string(),
        SKU: "1".to_string(),
        Condition: "3".to_string()
    };
    let shoe2 = Shoe{
        ShoeName: "Test".to_string(),
        Size: "11".to_string(),
        SKU: "1".to_string(),
        Condition: "3".to_string()
    };
    let o1 = Order{
        shipper: ShippingServices::DPD,
        tracking_number: "3212312".to_string(),
        shoe: shoe1.clone(),
        order_number: "132".to_string()
    };
    let o2 = Order{
        shipper: ShippingServices::DPD,
        tracking_number: "32112312".to_string(),
        shoe: shoe1.clone(),
        order_number: "333132".to_string()
    };
    let o3 = Order{
        shipper: ShippingServices::DPD,
        tracking_number: "321121312".to_string(),
        shoe: shoe2.clone(),
        order_number: "33123132".to_string()
    };
    let o4 = Order{
        shipper: ShippingServices::DPD,
        tracking_number: "32112312".to_string(),
        shoe: shoe1.clone(),
        order_number: "333132".to_string()
    };

    assert_eq!(vec![CountedShoes{ Shoe: shoe1.clone(), Number: 1 }], group_shoes(vec![o1.clone()]));
    assert_eq!(vec![CountedShoes{ Shoe: shoe1.clone(), Number: 2 }], group_shoes(vec![o1.clone(), o2.clone()]));
    assert_eq!(vec![CountedShoes{ Shoe: shoe1.clone(), Number: 3 }, CountedShoes{ Shoe: shoe2.clone(), Number: 1 }], group_shoes(vec![o1.clone(), o2.clone(), o3.clone(), o4.clone()]));

}


#[test]
fn test_OrderParsing(){
  let input_text = "1661 Inc
  Columbusstraat 25
  3165AC Rotterdam-Albrandswaard
  NL - NETHERLANDS
  
  Contact
  
  Phone
  Info
  
  Consignment 05212057104424\n\nRef1: Order 338109311DPDwww.dpd.nlSender account 348274Fabian Lukas BlankHauffstr. 37DE - 71093 Weil Im Schönbuch\nPackages\n1 of 1
  
  Weight
  13.05 Kg
  
  05212057 1044 24 A 2C RETURN
  
  Service
  NL-DPD-0521
  0521 B12
  332-NL-3165AC
  
  12/08/22 01:29-22070402-348274-shipper 2.3
  
  0316 5AC0 5212 0571 0442 4332 528A
   1\n\n338109311\n\nDunk Low 'UCLA'\n9 US M | DD1391 402 | New
  
  Ship by Mon 08/15
  
  DPD NL 05212057104424
  
  PLEASE INCLUDE WITH YOUR ITEM WHEN SHIPPING
  
  MCTSCHECKER
   1";
  
  let correct_result = Order{
    shipper: ShippingServices::DPD,
    tracking_number: "05212057104424".to_string(),
    shoe: Shoe { ShoeName: "Dunk Low 'UCLA'".to_string(), Size: "9 US M".to_string(), SKU: "DD1391 402".to_string(), Condition: "New".to_string() },
    order_number: "338109311".to_string(),
  };

  let res = parseTextToSale(input_text.to_string()).unwrap();
  assert_eq!(res, correct_result);
}
