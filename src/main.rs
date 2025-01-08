use dotenv::dotenv;
use poise::{
    serenity_prelude::{
        self as serenity, CreateAttachment, CreateEmbed, GetMessages, Message, MessageId,
    },
    CreateReply,
};
use std::io;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
/*
 Order Number: 12354
 Item Code: 62156
 Name: Azwad
 Address: Alexandria, Egypt
 Phone: 000000000000
 Price: 620
 Quantity: 2
*/
/// Add a company through a slash command
/// This is the least error prone way to add them to a csv
#[poise::command(slash_command)]
#[allow(clippy::too_many_arguments)]
async fn add_company(
    ctx: Context<'_>,
    #[description = "Order Number"] ordernumber: Option<String>,
    #[description = "Item Code"] itemcode: Option<String>,
    #[description = "Name"] name: Option<String>,
    #[description = "Address"] address: Option<String>,
    #[description = "Phone"] phone: Option<String>,
    #[description = "price"] price: Option<String>,
    #[description = "Quantity"] quantity: Option<String>,
) -> Result<(), Error> {
    if ordernumber.is_none()
        && name.is_none()
        && address.is_none()
        && phone.is_none()
        && quantity.is_none()
        && itemcode.is_none()
        && price.is_none()
    {
        return Err("No data to be added to the csv".into());
    }

    let id = ctx
        .guild_id()
        .map(|x| x.to_string())
        .unwrap_or(ctx.author().id.to_string());

    let file = match std::fs::File::options()
        .append(true)
        .create(true)
        .open(format!("slashcommands-{}.csv", id))
    {
        Ok(val) => val,
        Err(err) => return Err(format!("Couldn't open the csv file:\n{:#?}", err).into()),
    };
    let mut wtr = csv::Writer::from_writer(file);

    match wtr.serialize((
        ordernumber.clone(),
        itemcode.clone(),
        name.clone(),
        address.clone(),
        phone.clone(),
        price.clone(),
        quantity.clone(),
    )) {
        Ok(val) => val,
        Err(err) => return Err(format!("Couldn't write to the csv file:\n{:#?}", err).into()),
    };
    match wtr.flush() {
        Ok(val) => val,
        Err(err) => {
            return Err(format!(
                "Couldn't make sure everything was written to the csv file:\n{:#?}",
                err
            )
            .into())
        }
    };

    let reply = ctx.reply_builder(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Created a new record!")
                .description(format!(
                    "ordernumber: {}\nitem code: {}\nname: {}\naddress: {}\nphone: {}\nprice: {}\nquantity: {}",
                    ordernumber.unwrap_or_default(),
                    itemcode.unwrap_or_default(),
                    name.unwrap_or_default(),
                    address.unwrap_or_default(),
                    phone.unwrap_or_default(),
                    price.unwrap_or_default(),
                    quantity.unwrap_or_default()
                )),
        ),
    );
    ctx.send(reply).await?;
    Ok(())
}

/// Sends the CSV file from the commands used.
#[poise::command(slash_command)]
async fn get_csv(
    ctx: Context<'_>,
    #[description = "Delete CSV records after sending?"] delete: bool,
) -> Result<(), Error> {
    let id = ctx
        .guild_id()
        .map(|x| x.to_string())
        .unwrap_or(ctx.author().id.to_string());
    let pathid = format!("slashcommands-{}.csv", id.clone());
    let path = std::path::Path::new(&pathid);
    let attachment = match CreateAttachment::path(path).await {
        Ok(val) => val,
        Err(err) => {
            return Err(format!(
                "There is no csv available or could not open it:\n{:#?}",
                err
            )
            .into())
        }
    };
    let reply = ctx.reply_builder(CreateReply::default().attachment(attachment));
    ctx.send(reply).await?;
    if delete {
        std::fs::remove_file(path).unwrap();
    }
    Ok(())
}

///
#[poise::command(slash_command)]
async fn fetch_messages(
    ctx: Context<'_>,
    #[description = "Delete CSV records after sending? If not deleted, the next time fetches will be appended."]
    delete: bool,
    #[description = "What is the starting message? Please copy paste the message ID"]
    message_id: MessageId,
) -> Result<(), Error> {
    ctx.reply("Fetching messages").await?;
    let mut vecmessages: Vec<Message> = vec![];
    let mut lastmessageid = message_id;
    loop {
        let mut messages = ctx
            .channel_id()
            .messages(ctx, GetMessages::new().after(lastmessageid).limit(100))
            .await?;
        vecmessages.append(&mut messages.clone());
        if messages.len() < 100 {
            break;
        }
        println!("{}", vecmessages.len());
        lastmessageid = messages.first().unwrap().id;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    ctx.reply(format!(
        "Fetches {} messages, processing them now...",
        vecmessages.len()
    ))
    .await?;
    let id = ctx
        .guild_id()
        .map(|x| x.to_string())
        .unwrap_or(ctx.author().id.to_string());

    let file = match std::fs::File::options()
        .append(true)
        .create(true)
        .open(format!("fetch-{}.csv", id))
    {
        Ok(val) => val,
        Err(err) => return Err(format!("Couldn't open the csv file:\n{:#?}", err).into()),
    };
    let mut wtr = csv::Writer::from_writer(file);
    for message in vecmessages {
        let content = message.content.replace(":", "");
        /*
         Order Number: 12354
         Item Code: 62156
         Name: Azwad
         Address: Alexandria, Egypt
         Phone: 000000000000
         Price: 620
         Quantity: 2
        */
        let mut indexes = vec![];
        let mut ordernumbers = String::new();
        let mut itemcode = String::new();
        let mut name = String::new();
        let mut address = String::new();
        let mut phone = String::new();
        let mut price = String::new();
        let mut quantity = String::new();

        if let Some(index) = content.to_lowercase().find("order number") {
            indexes.push([index, index + 12]);
        }

        if let Some(index) = content.to_lowercase().find("item code") {
            indexes.push([index, index + 9]);
        }

        if let Some(index) = content.to_lowercase().find("name") {
            indexes.push([index, index + 4]);
        }

        if let Some(index) = content.to_lowercase().find("address") {
            indexes.push([index, index + 7]);
        }

        if let Some(index) = content.to_lowercase().find("phone") {
            indexes.push([index, index + 5]);
        }

        if let Some(index) = content.to_lowercase().find("price") {
            indexes.push([index, index + 5]);
        }

        if let Some(index) = content.to_lowercase().find("quantity") {
            indexes.push([index, index + 8]);
        }

        indexes.sort();
        println!("{:?}", indexes);
        if indexes.is_empty() {
            continue;
        }
        println!("CONTENT: {}", content);
        let mut count = 1;
        for index in indexes.clone() {
            let contentclone = content.clone();
            println!("{count}");
            let start = index[0];
            let end = index[1];
            let nextindex = if count >= indexes.len() {
                contentclone.len()
            } else {
                indexes[count][0]
            };
            count += 1;
            let line = contentclone.split_at(nextindex).0.split_at(start).1;
            let (mut variable, mut value) = line.split_at(end - start);
            variable = variable.trim().trim_end_matches(',');
            value = value.trim().trim_end_matches(',');

            match variable.to_lowercase().as_str() {
                "order number" => ordernumbers = value.to_string(),
                "item code" => itemcode = value.to_string(),
                "name" => name = value.to_string(),
                "address" => address = value.to_string(),
                "phone" => phone = value.to_string(),
                "price" => price = value.to_string(),
                "quantity" => quantity = value.to_string(),
                _ => continue,
            }
            println!(
                "ordernumber: {}\nitem code: {}\nname: {}\naddress: {}\nphone: {}\nprice: {}\nquantity: {}",
                ordernumbers,
                itemcode,
                name,
                address,
                phone,
                price,
                quantity
            )
        }

        match wtr.serialize((
            ordernumbers.clone(),
            itemcode.clone(),
            name.clone(),
            address.clone(),
            phone.clone(),
            price.clone(),
            quantity.clone(),
        )) {
            Ok(val) => val,
            Err(err) => return Err(format!("Couldn't write to the csv file:\n{:#?}", err).into()),
        };
    }
    match wtr.flush() {
        Ok(val) => val,
        Err(err) => {
            return Err(format!(
                "Couldn't make sure everything was written to the csv file:\n{:#?}",
                err
            )
            .into())
        }
    };
    let pathid = format!("fetch-{}.csv", id.clone());
    let path = std::path::Path::new(&pathid);

    let attachment = match CreateAttachment::path(path).await {
        Ok(val) => val,
        Err(err) => {
            return Err(format!(
                "There is no csv available or could not open it:\n{:#?}",
                err
            )
            .into())
        }
    };
    let reply = ctx.reply_builder(CreateReply::default().attachment(attachment));
    ctx.send(reply).await?;
    if delete {
        std::fs::remove_file(path).unwrap();
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().unwrap();
    let token = std::env::var("messagebot").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![add_company(), get_csv(), fetch_messages()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
