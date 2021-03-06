use serenity::client::CACHE;
use serenity::utils::Colour;
use serenity::framework::standard::CommandError;
use chrono;
use time::PreciseTime;
use cleverbot_io::Cleverbot;
use utils::logger;
use std::env;
use psutil;

command!(ping(_ctx, msg, _args) {
	let start = PreciseTime::now();
	let mut msg = match msg.channel_id.say("Pong!") {
		Ok(msg) => msg,
		Err(_) => return Ok(()),
	};
	let end = PreciseTime::now();
    let ms = start.to(end).num_milliseconds();

    let _ = msg.edit(|m| m.content(&format!("Pong!, **{}** ms", ms)));
});

command!(userinfo(_ctx, msg, _args) {
	if let Some(guild) = msg.guild() {
		let guild = guild.read();
      
		if let Some(member) = guild.members.get(&msg.author.id) {
			let mut roles = "@every\u{200b}one".to_owned();
			let mut iter = member.roles.iter();
			while let Some(role_id) = iter.next() {
				if let Some(role) = role_id.find() {
					roles.push_str(", ");
					roles.push_str(&role.name);
				} else {
					return Err(CommandError::from("No RoleId for this Role".to_string()));
				}
			}

			let joined_at = {
				if let Some(join_date) = member.joined_at.as_ref() {
					join_date.naive_utc().format("%c")
				} else {
					chrono::naive::NaiveDateTime::from_timestamp(0, 0).format("%c") 
				}
			};
			let avatar_url = msg.author.face();
			let id = msg.author.id.0.to_string();
			let nick = member.nick.as_ref().unwrap_or_else(|| &msg.author.name);
			let dtag = msg.author.tag();
			let created_at = msg.author.created_at().format("%c").to_string();
			let footer_text = format!("Member since {}", joined_at);

			let _ = match msg.channel_id.send_message(|cm| cm.embed(|ce|
				ce.author(|cea| cea.name(&dtag).icon_url(&avatar_url))
					.title("Info")
					.field("ID", &id, true)
					.field("Current Name", nick, true)
					.field("Created at", &created_at, true)
					.field("Roles", &roles, true)
					.footer(|cef| cef.text(&footer_text))
					.image(&avatar_url)
					.color(Colour::from_rgb(246, 219, 216))
			)){
				Ok(msg) => msg,
				Err(why) => {
					logger::error(format!("{:?}", why));
					let _ = msg.channel_id.say(format!("Error sending embed:\n{:?}", why));
					return Ok(());
				},
			};
    	}
  	}
});

command!(cleverbot(_ctx, msg, args) {
	if args.is_empty() {
		let _ = msg.channel_id.say("What do you want to talk about?");
		return Ok(());
	}

	let query = args.full();

	let _ = msg.channel_id.broadcast_typing();

	let mut bot = Cleverbot::new(env::var("CLEVERBOT_USER").unwrap(), env::var("CLEVERBOT_KEY").unwrap(), None).unwrap();
	let res = bot.say(&query).unwrap();
	let m = msg.channel_id.say(res);
	if m.is_err() {
		logger::error(format!("Error sending message\n{:?}", m));
		return Ok(());
	}
});

command!(stats(_ctx, msg, _args) {
    let processes = match psutil::process::all() {
		Ok(processes) => processes,
		Err(why) => {
			println!("Err getting processes: {:?}", why);
			let _ = msg.channel_id.say("Error getting stats");
			return Ok(());
		},
  	};

  	let process = match processes.iter().find(|p| p.pid == psutil::getpid()) {
		Some(process) => process,
		None => {
			let _ = msg.channel_id.say("Error getting stats");
			return Ok(());
		},
  	};

	let memory = match process.memory() {
		Ok(memory) => memory,
		Err(why) => {
			println!("Err getting process memory: {:?}", why);
			let _ = msg.channel_id.say("Error getting stats");
			return Ok(());
		},
	};

  	const B_TO_MB: u64 = 1024 * 1024;
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	let mem_total = memory.size / B_TO_MB;
	let guilds = CACHE.read().guilds.len();
	let channels = CACHE.read().channels.len();
	let users = CACHE.read().users.len();
	let cur_guild_id = msg.guild_id().unwrap();
	let cur_shard = cur_guild_id.shard_id();

	let _ = msg.channel_id.send_message(|m| m
		.embed(|e| e
			.color(Colour::from_rgb(246, 219, 216))
			.title("Statistics")
			.field("Version", &VERSION, true)
			.field("Memory Used", &format!("{}MB", mem_total), true)
			.field("Shard", &format!("{}/1", cur_shard.to_string()), true)
			.field("Guilds", &guilds.to_string(), true)
			.field("Users", &users.to_string(), true)
			.field("Channels", &channels.to_string(), true)));
});

command!(invite(_ctx, msg, _args) {
	let _ = msg.channel_id.say("You can invite me with: **<https://discordapp.com/oauth2/authorize?&client_id=380101459062161409&scope=bot&permissions=66186303>**");
});