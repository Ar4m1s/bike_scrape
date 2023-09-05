use regex::Regex;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let email = &args[1];
    let password = &args[2];
    let file_name = &args[3];

    let client = reqwest::Client::builder().cookie_store(true).build()?;
    let overview = download_order_overview(&client, email.to_owned(), password.to_owned()).await?;

    let ids = get_ids(overview);
    let prefix = "<hr></hr>";

    std::fs::write(file_name, "<h1>Oversikt over kjøp<h1>".to_owned() + prefix)?;

    for id in ids {
        let a = download_order(&client, id).await?;
        let table = parse_order(a);
        std::fs::write(
            file_name,
            std::fs::read_to_string(file_name)? + &table.join("\n") + "</table>\n" + prefix,
        )?;
    }

    Ok(())
}

fn parse_order(text: String) -> Vec<String> {
    let regex = Regex::new(r#"(?ms)LabelOrdrenr">(\d*).*LabelRegDato">([\d|\.]*).*(<table id="ProductList".*)<table width.*Blå"#).unwrap();
    let mut data = vec![];
    for i in 1..4 {
        data.push(
            regex
                .captures(&text)
                .unwrap()
                .get(i)
                .map_or("", |m| m.as_str())
                .to_string(),
        );
    }
    data[0] = "\n<h2>Ordrenummer: ".to_owned() + &data[0] + "</h2>";
    data[1] = "<h2>Dato: ".to_owned() + &data[1] + "</h2>";
    data
}

fn get_ids(text: String) -> Vec<String> {
    let regex = Regex::new(r"(?m)Ordrenr:</span>([\d]*)\s").expect("failed to parse ordreoversikt");
    let result = regex.captures_iter(&text);
    let mut ids = vec![];

    for mat in result {
        ids.push(String::from(mat.get(1).map_or("", |m| m.as_str())).to_string());
    }
    ids
}

async fn download_order(client: &Client, id: String) -> Result<String, Box<dyn std::error::Error>> {
    let base = "https://bikeshop.no/kundesenter/ordrehistorikk/ordrestatus/ordre?ordrenr=";
    let response = client
        .get(base.to_owned() + &id)
        .send()
        .await?
        .text()
        .await?;
    Ok(response)
}

async fn download_order_overview(
    client: &Client,
    email: String,
    password: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let params = [("username", email), ("password", password)];

    client
        .post("https://bikeshop.no/api/Login/Login")
        .form(&params)
        .send()
        .await?
        .text()
        .await?;

    let params = [
        ("__EVENTTARGET", "ctl00$CPHCnt$Ordreliste1$GaiaRadioVisAlle"),
        (
            "ctl00$CPHCnt$Ordreliste1$GaiaRadioVisAlle",
            "GaiaRadioVisAlle",
        ),
    ];

    let resp = client
        .post("https://bikeshop.no/kundesenter/ordrehistorikk/ordrestatus")
        .form(&params)
        .send()
        .await?
        .text()
        .await?;
    Ok(resp)
}
