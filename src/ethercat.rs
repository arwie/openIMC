use ethercat::{
    AlState, DomainIdx, Idx, Master, MasterAccess, Offset, PdoCfg, PdoEntryIdx,
    PdoEntryInfo, PdoEntryPos, PdoIdx, SdoIdx, SlaveAddr, SlaveId, SlavePos,
    SmCfg, SubIdx,
};
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use std::{thread, time};
use std::time::{Duration, Instant};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};


#[derive(Debug, Clone, Copy)]
struct PdoOffsets {
	status: u64,
	control: u64,
	pfb: u64,
	pcmd: u64,
}

struct Ethercat {
	master: Master,
	domain_idx: DomainIdx,
	offsets: PdoOffsets,
	start_time: Instant,
}

static ETHERCAT: OnceCell<Mutex<Ethercat>> = OnceCell::new();
static PFBZERO: OnceCell<i32> = OnceCell::new();


pub fn init() -> () {

	let mut master = Master::open(0, MasterAccess::ReadWrite).unwrap();
	master.reserve().unwrap();
	
	let domain_idx = master.create_domain().unwrap();
	
	let mut config = master.configure_slave(
		SlaveAddr::ByPos(0),
		SlaveId {
			vendor_id: 0x000002e1,
			product_code: 0x00000000,
		}
	).unwrap();
	
	let offsets = PdoOffsets {
		status: config.register_pdo_entry(
				PdoEntryIdx::new(0x6041, 0),
				domain_idx
			).unwrap().byte as u64,
		control: config.register_pdo_entry(
				PdoEntryIdx::new(0x6040, 0),
				domain_idx
			).unwrap().byte as u64,
		pfb: config.register_pdo_entry(
				PdoEntryIdx::new(0x6064, 0),
				domain_idx
			).unwrap().byte as u64,
		pcmd: config.register_pdo_entry(
				PdoEntryIdx::new(0x607A, 0),
				domain_idx
			).unwrap().byte as u64,
	};

	config.config_dc(0x0700, 4000000, 0, 0, 0).unwrap();
	
	master.sdo_download(SlavePos::new(0), SdoIdx::new(0x6040, 0), false, &0x80u16).unwrap();	//reset faluts
	master.sdo_download(SlavePos::new(0), SdoIdx::new(0x6060, 0), false, &8u8).unwrap();		//Modes of Operation: 8 [cyclic synchronous position mode]

	master.sdo_download(SlavePos::new(0), SdoIdx::new(0x60C2, 1), false, &40u8).unwrap();
	master.sdo_download(SlavePos::new(0), SdoIdx::new(0x60C2, 2), false, &-4i8).unwrap();

	thread::sleep(time::Duration::from_millis(100));
	
	master.activate().unwrap();
	
	ETHERCAT.set(Mutex::new(Ethercat {
		master,
		domain_idx,
		offsets,
		start_time: Instant::now(),
	}));
}


pub fn process(pos:f64) ->() {
	let mut ec = ETHERCAT.get().unwrap().lock().unwrap();
	let domain_idx = ec.domain_idx;
	let offsets = ec.offsets;
	let start_time = ec.start_time;
	
	ec.master.receive().unwrap();
	ec.master.domain(ec.domain_idx).process().unwrap();
	
	ec.master.set_application_time(start_time.elapsed().as_nanos() as u64).unwrap();
	ec.master.sync_reference_clock().unwrap();
	ec.master.sync_slave_clocks().unwrap();
	
	let mut data = Cursor::new(ec.master.domain_data(domain_idx).unwrap());
	
	data.set_position(offsets.pfb);
	let pfb = data.read_i32::<LittleEndian>().unwrap();
	
	data.set_position(offsets.status);
	let status = data.read_u16::<LittleEndian>().unwrap();
	
	
	let mut control: u16 = 0;
	let mut pcmd = pfb;
	
	match status & 0b111 {
		0b000 => control = 0x06,
		0b001 => control = 0x07,
		0b011 => control = 0x0F,
		0b111 => {
				control = 0x1F;
				pcmd = ((pos * 20000.0) as i32) - PFBZERO.get_or_init(|| pfb);
			},
		_ => (),
	}
	
	data.set_position(offsets.control);
	data.write_u16::<LittleEndian>(control).unwrap();
	
	data.set_position(offsets.pcmd);
	data.write_i32::<LittleEndian>(pcmd).unwrap();
	
	
	//print!("status:{:#016b}  ", status);
	//print!("control:{:#016b}  ", control);
	//print!("pos:{:?}  ", pos);
	//print!("pfb:{:?}  ", pfb);
	//print!("pcmd:{:?}  ", pcmd);
	//println!();
	
	ec.master.domain(ec.domain_idx).queue().unwrap();
	ec.master.send().unwrap();
}
