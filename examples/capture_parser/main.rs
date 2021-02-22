mod zip_context;

use std::collections::HashMap;
use std::env;
use std::io::{BufReader, Result as Res};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::time::Instant;

use lu_packets::{
	auth::server::Message as AuthServerMessage,
	world::Lot,
	world::server::Message as WorldServerMessage,
	world::client::Message as WorldClientMessage,
};
use rusqlite::{params, Connection};
use zip::ZipArchive;
use self::zip_context::ZipContext;

static mut PRINT_PACKETS: bool = false;

const COMP_ORDER : [u32; 20] = [1, 3, 40, 98, 7, 23, 110, 109, 106, 4, 17, 5, 9, 60, 48, 16, 6, 2, 44, 107];

pub struct Cdclient {
	conn: Connection,
	comp_cache: HashMap<Lot, Vec<u32>>,
}

impl Cdclient {
	fn get_comps(&mut self, lot: Lot) -> &Vec<u32> {
		if !self.comp_cache.contains_key(&lot) {
			let mut stmt = self.conn.prepare("select component_type from componentsregistry where id = ?").unwrap();
			let rows = stmt.query_map(params![lot], |row| row.get(0)).unwrap();
			let mut comps = vec![];
			for row in rows {
				let value = row.unwrap();
				comps.push(value);
				// special case: implied components
				match value {
					2  => { comps.push(44); }
					4  => { comps.push(110); comps.push(109); comps.push(106); }
					7  => { comps.push(98); }
					23 => { comps.push(7); }
					48 => { comps.push(7); }
					_ => {},
				}
			}
			dbg!(&comps);
			comps.sort();
			comps.dedup();
			dbg!(&comps);
			comps.sort_by_key(|x| COMP_ORDER.iter().position(|y| y == x).unwrap_or(usize::MAX));
			dbg!(&comps);
			self.comp_cache.insert(lot, comps);
		}
		&self.comp_cache.get(&lot).unwrap()
	}
}

fn visit_dirs(dir: &Path, cdclient: &mut Cdclient, level: usize) -> Res<usize> {
	let mut packet_count = 0;
	if dir.is_dir() {
		for entry in fs::read_dir(dir)? {
			let entry = entry?;
			let path = entry.path();
			packet_count += if path.is_dir() { visit_dirs(&path, cdclient, level+1) } else { parse(&path, cdclient) }?;
			println!("packet count = {:>level$}", packet_count, level=level*6);
		}
	}
	Ok(packet_count)
}

fn parse(path: &Path, cdclient: &mut Cdclient) -> Res<usize> {
	use endio::LERead;

	if path.extension().unwrap() != "zip" { return Ok(0); }

	let src = BufReader::new(File::open(path).unwrap());
	let mut zip = ZipArchive::new(src).unwrap();
	let mut lots = HashMap::new();
	let mut i = 0;
	let mut packet_count = 0;
	while i < zip.len() {
		let mut file = zip.by_index(i).unwrap();
		if file.name().contains("of") {
			i += 1; continue;
		}
		if file.name().contains("[53-01-") {
			let msg: AuthServerMessage = file.read().expect(&format!("Zip: {}, Filename: {}, {} bytes", path.to_str().unwrap(), file.name(), file.size()));
			if unsafe { PRINT_PACKETS } {
				dbg!(msg);
			}
			packet_count += 1
		} else if file.name().contains("[53-04-")
			&& !file.name().contains("[53-04-00-16]")
			&& !file.name().contains("[e6-00]")
			&& !file.name().contains("[6b-03]")
			&& !file.name().contains("[16-04]")
			&& !file.name().contains("[49-04]")
			&& !file.name().contains("[ad-04]")
			&& !file.name().contains("[1c-05]")
			&& !file.name().contains("[230]")
			&& !file.name().contains("[875]")
			&& !file.name().contains("[1046]")
			&& !file.name().contains("[1097]")
			&& !file.name().contains("[1197]")
			&& !file.name().contains("[1308]")
		{
			let msg: WorldServerMessage = file.read().expect(&format!("Zip: {}, Filename: {}, {} bytes", path.to_str().unwrap(), file.name(), file.size()));
			if unsafe { PRINT_PACKETS } {
				dbg!(&msg);
			}
			packet_count += 1;
		} else if file.name().contains("[53-02-") || (file.name().contains("[53-05-")
		&& !file.name().contains("[53-05-00-00]")
		&& !file.name().contains("[53-05-00-15]")
		&& !file.name().contains("[53-05-00-31]")
		&& !file.name().contains("[76-00]")
		&& !file.name().contains("[e6-00]")
		&& !file.name().contains("[ff-00]")
		&& !file.name().contains("[a1-01]")
		&& !file.name().contains("[7f-02]")
		&& !file.name().contains("[a3-02]")
		&& !file.name().contains("[cc-02]")
		&& !file.name().contains("[35-03]")
		&& !file.name().contains("[36-03]")
		&& !file.name().contains("[4d-03]")
		&& !file.name().contains("[6d-03]")
		&& !file.name().contains("[91-03]")
		&& !file.name().contains("[1a-05]")
		&& !file.name().contains("[e6-05]")
		&& !file.name().contains("[16-06]")
		&& !file.name().contains("[1c-06]")
		&& !file.name().contains("[6f-06]")
		&& !file.name().contains("[70-06]")
		&& !file.name().contains("[118]")
		&& !file.name().contains("[230]")
		&& !file.name().contains("[255]")
		&& !file.name().contains("[417]")
		&& !file.name().contains("[639]")
		&& !file.name().contains("[675]")
		&& !file.name().contains("[716]")
		&& !file.name().contains("[821]")
		&& !file.name().contains("[822]")
		&& !file.name().contains("[845]")
		&& !file.name().contains("[877]")
		&& !file.name().contains("[913]")
		&& !file.name().contains("[1306]")
		&& !file.name().contains("[1510]")
		&& !file.name().contains("[1558]")
		&& !file.name().contains("[1564]")
		&& !file.name().contains("[1647]")
		&& !file.name().contains("[1648]"))
		|| (file.name().contains("[24]")
		&& !file.name().contains("(2365)")
		&& !file.name().contains("(4930)")
		&& !file.name().contains("(5635)")
		&& !file.name().contains("(5958)")
		&& !file.name().contains("(6007)")
		&& !file.name().contains("(6010)")
		&& !file.name().contains("(6209)")
		&& !file.name().contains("(6267)")
		&& !file.name().contains("(6289)")
		&& !file.name().contains("(6319)")
		&& !file.name().contains("(7282)")
		&& !file.name().contains("(8304)"))
		|| file.name().contains("[27]")
		{
			let mut ctx = ZipContext { zip: file, lots: &mut lots, cdclient, assert_fully_read: true };
			let msg: WorldClientMessage = ctx.read().expect(&format!("Zip: {}, Filename: {}, {} bytes", path.to_str().unwrap(), ctx.zip.name(), ctx.zip.size()));
			file = ctx.zip;
			if unsafe { PRINT_PACKETS } {
				dbg!(&msg);
			}
			packet_count += 1;

			if ctx.assert_fully_read {
				// assert fully read
				let mut rest = vec![];
				std::io::Read::read_to_end(&mut file, &mut rest).unwrap();
				assert_eq!(rest, vec![], "Zip: {}, Filename: {}, {} bytes", path.to_str().unwrap(), file.name(), file.size());
			}
			i += 1; continue
		} else { i += 1; continue }
		// assert fully read
		let mut rest = vec![];
		std::io::Read::read_to_end(&mut file, &mut rest).unwrap();
		assert_eq!(rest, vec![], "Zip: {}, Filename: {}, {} bytes", path.to_str().unwrap(), file.name(), file.size());
		i += 1;
	}
	Ok(packet_count)
}

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 3 {
		println!("Usage: capture_parser capture_path cdclient_path --print_packets");
		return;
	}
	let capture = fs::canonicalize(&args[1]).unwrap();
	let mut cdclient = Cdclient { conn: Connection::open(&args[2]).unwrap(), comp_cache: HashMap::new() };
	unsafe { PRINT_PACKETS = args.get(3).is_some(); }

	let start = Instant::now();
	let packet_count = if capture.ends_with(".zip") {
		parse(&capture, &mut cdclient)
	} else {
		visit_dirs(&capture, &mut cdclient, 0)
	}.unwrap();
	println!();
	println!("Number of parsed packets: {}", packet_count);
	println!("Time taken: {:?}", start.elapsed());
}
