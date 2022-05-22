use std::{fs::File, path::PathBuf};

use clap::Parser;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Sequence, Value};

// Constants
const DEFAULT_AMOUNT: f64 = 1.0;
const DEFAULT_UNIT: Unit = Unit::Hours;
const DEFAULT_UNIT_PRICE: f64 = 50.0;

struct Defaults {}

impl Defaults {
    const fn get_default_amount() -> f64 {
        DEFAULT_AMOUNT
    }

    const fn get_default_unit() -> Unit {
        DEFAULT_UNIT
    }

    const fn get_default_unit_price() -> f64 {
        DEFAULT_UNIT_PRICE
    }
}

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
    let mut invoice: Invoice =
        serde_yaml::from_reader(f).expect("Error: Failed to deserialize invoice from file");
    invoice.compute();
    let invoice = invoice;

    // println!("{:?}", invoice);

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

#[derive(Debug, Serialize, Deserialize)]
struct Invoice {
    number: String,
    issued: String,
    shipped: String,

    company: Company,
    client: Client,

    detail: Vec<Product>,

    #[serde(default)]
    total: f64,

    payment: Payment,
}

impl Invoice {
    fn compute(&mut self) {
        self.detail.iter_mut().for_each(Product::compute);
        self.total = self.detail.iter().map(|p| p.total).sum();
    }

    // fn from_data(data: InvoiceData) -> Self {
    //     Self {
    //         number: data.number,
    //         issued: data.issued,
    //         shipped: data.shipped,

    //         company: Company::from_mapping(&data.company),
    //         client: Client::from_mapping(&data.client),

    //         detail: Product::vec_from_sequence(&data.detail),

    //         payment: Payment::from_mapping(&data.payment),
    //     }
    // }
}

#[derive(Debug, Serialize, Deserialize)]
struct Company {
    name: String,
    activity: String,
    address: Vec<String>,
    phone: Phone,
    website: String,
    mail: String,
    siret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Client {
    name: String,
    address: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Phone {
    display: String,
    intl: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Product {
    description: String,
    #[serde(default = "Defaults::get_default_amount")]
    amount: f64,
    #[serde(default = "Defaults::get_default_unit")]
    unit: Unit,
    #[serde(default = "Defaults::get_default_unit_price")]
    unit_price: f64,
    #[serde(default)]
    total: f64,
}

impl Product {
    fn compute(&mut self) {
        self.total = self.amount * self.unit_price;
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Unit {
    #[serde(rename(serialize = "Heures", deserialize = "hours"), alias = "hours")]
    Hours,
    #[serde(rename(serialize = "Unités", deserialize = "units"), alias = "units")]
    Units,
}

#[derive(Debug, Serialize, Deserialize)]
struct Payment {
    r#type: PaymentType,
    delay: u64,
    penalty: f64,
}

#[derive(Debug, Serialize, Deserialize)]
enum PaymentType {
    #[serde(
        rename(serialize = "Virement", deserialize = "virement"),
        alias = "virement"
    )]
    Virement,
    #[serde(rename(serialize = "Carte bancaire", deserialize = "cb"), alias = "cb")]
    CarteBancaire,
    #[serde(rename(serialize = "Espèce", deserialize = "espece"), alias = "espece")]
    Espece,
    #[serde(rename(serialize = "Chèque", deserialize = "cheque"), alias = "cheque")]
    Cheque,
}
