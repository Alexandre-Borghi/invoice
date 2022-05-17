use std::{fs::File, ops::Deref, path::PathBuf};

use clap::Parser;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Sequence, Value};

/// Generates invoices for my freelance activity
#[derive(Parser, Debug)]
#[clap(author = "Alexandre Borghi <borghi.alexandre.12@gmail.com>")]
#[clap(version, about, long_about = None)]
struct Args {
    /// Path to the invoice data YAML file
    path: PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let f = File::open(args.path)?;
    let invoice: InvoiceData =
        serde_yaml::from_reader(f).expect("Error: Failed to deserialize invoice from file");

    println!("{:?}", invoice);

    let mut handlebars = Handlebars::new();

    handlebars
        .register_template_file("invoice", "invoice.html.hbs")
        .expect("Failed to register template file");

    let html_path = format!("invoice-{}.html", invoice.number);
    let pdf_path = format!("invoice-{}.pdf", invoice.number);

    let mut html_output = File::create(&html_path).expect("Failed to create html output file");
    handlebars
        .render_to_write("invoice", &invoice, html_output)
        .expect("Failed to render template");

    println!("[INFO] {} generated successfully", &html_path);

    Ok(())
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct InvoiceData {
    number: String,
    issued: String,  // date
    shipped: String, // date

    company: Mapping,
    client: Mapping,

    detail: Sequence,

    payment: Mapping,
}

struct Invoice {
    number: String,
    issued: String,
    shipped: String,

    company: Company,
    client: Client,

    detail: Vec<Product>,

    payment: Payment,
}

impl Invoice {
    fn from_data(data: InvoiceData) -> Self {
        Self {
            number: data.number,
            issued: data.issued,
            shipped: data.shipped,

            company: Company::from_mapping(&data.company),
            client: Client::from_mapping(&data.client),

            detail: Product::vec_from_sequence(&data.detail),

            payment: Payment::from_mapping(&data.payment),
        }
    }
}

struct Company {
    name: String,
    activity: String,
    address: Vec<String>,
    phone: Phone,
    website: String,
    mail: String,
    siret: String,
}

impl Company {
    fn from_mapping(mapping: &Mapping) -> Self {
        Self {
            name: get_string_from_mapping(mapping, "name"),
            activity: get_string_from_mapping(mapping, "activity"),
            address: string_vec_from_sequence(
                mapping
                    .get(&Value::from("activity"))
                    .expect("expected value for key `address`")
                    .as_sequence()
                    .expect("expected `address` to be a sequence"),
            ),
            phone: Phone::from_mapping(
                mapping
                    .get(&Value::from("phone"))
                    .expect("expected value for key `phone`")
                    .as_mapping()
                    .expect("expected value for `phone` to be a mapping"),
            ),
            website: get_string_from_mapping(mapping, "website"),
            mail: get_string_from_mapping(mapping, "mail"),
            siret: get_string_from_mapping(mapping, "siret"),
        }
    }
}

struct Client {
    name: String,
    address: Vec<String>,
}

impl Client {
    fn from_mapping(mapping: &Mapping) -> Self {
        Self {
            name: get_string_from_mapping(mapping, "name"),
            address: string_vec_from_sequence(
                mapping
                    .get(&Value::from("address"))
                    .expect("expected value for key `address`")
                    .as_sequence()
                    .expect("expected value for `address` to be a sequence"),
            ),
        }
    }
}

struct Phone {
    display: String,
    intl: String,
}

impl Phone {
    fn from_mapping(mapping: &Mapping) -> Self {
        Self {
            display: get_string_from_mapping(mapping, "display"),
            intl: get_string_from_mapping(mapping, "intl"),
        }
    }
}

struct Product {
    description: String,
    amount: f64,
    unit: Unit,
    unit_price: f64,
}

impl Product {
    fn from_mapping(mapping: &Mapping) -> Self {
        Self {
            description: get_string_from_mapping(mapping, "description"),
            amount: get_f64_from_mapping(mapping, "amount"),
            unit: Unit::from_str(&get_string_from_mapping(mapping, "unit")).unwrap_or(Unit::Hours),
            unit_price: get_f64_from_mapping(mapping, "unit_price"),
        }
    }

    fn vec_from_sequence(sequence: &Sequence) -> Vec<Self> {
        sequence
            .into_iter()
            .map(|v| {
                v.as_mapping()
                    .expect("expected value for element of `detail` to be a mapping")
            })
            .map(Product::from_mapping)
            .collect()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Unit {
    #[serde(rename = "hours")]
    Hours,
    #[serde(rename = "units")]
    Units,
}

impl Unit {
    fn from_str(string: &str) -> Option<Self> {
        match string {
            "hours" => Some(Self::Hours),
            "units" => Some(Self::Units),
            _ => None,
        }
    }
}

struct Payment {
    r#type: PaymentType,
    delay: u64,
}

impl Payment {
    fn from_mapping(mapping: &Mapping) -> Self {
        Self {
            r#type: PaymentType::from_str(&get_string_from_mapping(mapping, "type"))
                .expect("expected value for key `type` to be a valid PaymentType variant"),
            delay: get_u64_from_mapping(mapping, "delay"),
        }
    }
}

enum PaymentType {
    Virement,
    CarteBancaire,
    Espece,
    Cheque,
}

impl PaymentType {
    fn from_str(string: &str) -> Option<Self> {
        match string {
            "virement" => Some(Self::Virement),
            "cb" => Some(Self::CarteBancaire),
            "espece" => Some(Self::Espece),
            "cheque" => Some(Self::Cheque),
            _ => None,
        }
    }
}

fn get_string_from_mapping(mapping: &Mapping, key: &str) -> String {
    let val = get_value_from_mapping(mapping, key);
    val.as_str()
        .expect(format!("value {val:?} for key `{key}` couldn't be converted to a string").as_str())
        .to_string()
}

fn get_u64_from_mapping(mapping: &Mapping, key: &str) -> u64 {
    let val = get_value_from_mapping(mapping, key);
    val.as_u64()
        .expect(format!("value {val:?} for key `{key}` couldn't be converted to a u64").as_str())
}

fn get_f64_from_mapping(mapping: &Mapping, key: &str) -> f64 {
    let val = get_value_from_mapping(mapping, key);
    val.as_f64()
        .expect(format!("value {val:?} for key `{key}` couldn't be converted to an f64").as_str())
}

fn get_value_from_mapping(mapping: &Mapping, key: &str) -> Value {
    mapping
        .get(&Value::from(key))
        .expect(format!("expected value for key `{key}`").as_str())
        .to_owned()
}

fn string_vec_from_sequence(sequence: &Sequence) -> Vec<String> {
    sequence
        .into_iter()
        .map(|v| {
            v.as_str()
                .expect(
                    format!("expected element of `address` to be a string, instead found {v:?}")
                        .as_str(),
                )
                .to_string()
        })
        .collect()
}
