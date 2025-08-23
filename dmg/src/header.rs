use std::string::String;
use crate::gb::GameBoy;
use crate::mbc;
#[derive(Debug)]
enum Destination {
    Japan,
    RestOfWorld,
    Undefined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewLicensee {
    None,                      // "00"
    NintendoRD1,              // "01"
    Capcom,                   // "08"
    ElectronicArts,           // "13", "69"
    HudsonSoft,               // "18", "38"
    BAI,                      // "19"
    KSS,                      // "20"
    PlanningOfficeWADA,       // "22"
    PCMComplete,              // "24"
    SanX,                     // "25"
    Kemco,                    // "28"
    SETACorporation,          // "29"
    Viacom,                   // "30"
    Nintendo,                 // "31"
    Bandai,                   // "32"
    OceanSoftwareAcclaim,     // "33", "67", "93"
    Konami,                   // "34", "54", "A4"
    HectorSoft,               // "35"
    Taito,                    // "37"
    Banpresto,                // "39"
    UbiSoft,                  // "41"
    Atlus,                    // "42"
    MalibuInteractive,        // "44"
    Angel,                    // "46"
    BulletProofSoftware,      // "47"
    Irem,                     // "49"
    Absolute,                 // "50"
    AcclaimEntertainment,     // "51"
    Activision,               // "52"
    SammyUSACorporation,      // "53"
    HiTechExpressions,        // "55"
    LJN,                      // "56"
    Matchbox,                 // "57"
    Mattel,                   // "58"
    MiltonBradleyCompany,     // "59"
    TitusInteractive,         // "60"
    VirginGames,              // "61"
    LucasfilmGames,           // "64"
    Infogrames,               // "70"
    InterplayEntertainment,   // "71"
    Broderbund,               // "72"
    SculpturedSoftware,       // "73"
    TheSalesCurveLimited,     // "75"
    THQ,                      // "78"
    Accolade,                 // "79"
    MisawaEntertainment,      // "80"
    Lozc,                     // "83"
    TokumaShoten,             // "86"
    TsukudaOriginal,          // "87"
    ChunsoftCo,               // "91"
    VideoSystem,              // "92"
    Varie,                    // "95"
    YonezawaSpal,             // "96"
    Kaneko,                   // "97"
    PackInVideo,              // "99"
    BottomUp,                 // "9H"
    MTO,                      // "BL"
    Kodansha,                 // "DK"
    Unknown,
}

impl NewLicensee {
    /// Convert from a two-character ASCII string to a NewLicensee enum variant
    pub fn from_ascii_code(code: &str) -> Self {
        match code {
            "00" => Self::None,
            "01" => Self::NintendoRD1,
            "08" => Self::Capcom,
            "13" | "69" => Self::ElectronicArts,
            "18" | "38" => Self::HudsonSoft,
            "19" => Self::BAI,
            "20" => Self::KSS,
            "22" => Self::PlanningOfficeWADA,
            "24" => Self::PCMComplete,
            "25" => Self::SanX,
            "28" => Self::Kemco,
            "29" => Self::SETACorporation,
            "30" => Self::Viacom,
            "31" => Self::Nintendo,
            "32" => Self::Bandai,
            "33" | "67" | "93" => Self::OceanSoftwareAcclaim,
            "34" | "54" | "A4" => Self::Konami,
            "35" => Self::HectorSoft,
            "37" => Self::Taito,
            "39" => Self::Banpresto,
            "41" => Self::UbiSoft,
            "42" => Self::Atlus,
            "44" => Self::MalibuInteractive,
            "46" => Self::Angel,
            "47" => Self::BulletProofSoftware,
            "49" => Self::Irem,
            "50" => Self::Absolute,
            "51" => Self::AcclaimEntertainment,
            "52" => Self::Activision,
            "53" => Self::SammyUSACorporation,
            "55" => Self::HiTechExpressions,
            "56" => Self::LJN,
            "57" => Self::Matchbox,
            "58" => Self::Mattel,
            "59" => Self::MiltonBradleyCompany,
            "60" => Self::TitusInteractive,
            "61" => Self::VirginGames,
            "64" => Self::LucasfilmGames,
            "70" => Self::Infogrames,
            "71" => Self::InterplayEntertainment,
            "72" => Self::Broderbund,
            "73" => Self::SculpturedSoftware,
            "75" => Self::TheSalesCurveLimited,
            "78" => Self::THQ,
            "79" => Self::Accolade,
            "80" => Self::MisawaEntertainment,
            "83" => Self::Lozc,
            "86" => Self::TokumaShoten,
            "87" => Self::TsukudaOriginal,
            "91" => Self::ChunsoftCo,
            "92" => Self::VideoSystem,
            "95" => Self::Varie,
            "96" => Self::YonezawaSpal,
            "97" => Self::Kaneko,
            "99" => Self::PackInVideo,
            "9H" => Self::BottomUp,
            "BL" => Self::MTO,
            "DK" => Self::Kodansha,
            _ => Self::Unknown,
        }
    }

    /// Convert from bytes read from a file/header
    pub fn from_bytes(bytes: &[u8; 2]) -> Self {
        // Convert bytes to ASCII string
        let ascii_string = String::from_utf8(bytes.to_vec()).unwrap();
        Self::from_ascii_code(&ascii_string)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OldLicensee {
    None,                           // 0x00
    Nintendo,                       // 0x01, 0x31
    Capcom,                         // 0x08, 0x38
    HOTB,                          // 0x09
    Jaleco,                        // 0x0A, 0xE0
    CoconutsJapan,                 // 0x0B
    EliteSystems,                  // 0x0C, 0x6E
    ElectronicArts,                // 0x13, 0x69
    HudsonSoft,                    // 0x18
    ITCEntertainment,              // 0x19
    Yanoman,                       // 0x1A
    JapanClary,                    // 0x1D
    VirginGames,                   // 0x1F, 0x4A, 0x61
    PCMComplete,                   // 0x24
    SanX,                          // 0x25
    Kemco,                         // 0x28, 0x7F, 0x97, 0xC2
    SETACorporation,               // 0x29
    Infogrames,                    // 0x30, 0x70
    Bandai,                        // 0x32, 0xA2, 0xB2
    UseNewLicenseeCode,            // 0x33
    Konami,                        // 0x34, 0xA4
    HectorSoft,                    // 0x35
    Banpresto,                     // 0x39, 0x9D, 0xD9
    EntertainmentInteractive,      // 0x3C
    Gremlin,                       // 0x3E
    UbiSoft,                       // 0x41
    Atlus,                         // 0x42, 0xEB
    MalibuInteractive,             // 0x44, 0x4D
    Angel,                         // 0x46, 0xCF
    SpectrumHoloByte,              // 0x47
    Irem,                          // 0x49
    USGold,                        // 0x4F
    Absolute,                      // 0x50
    AcclaimEntertainment,          // 0x51, 0xB0
    Activision,                    // 0x52
    SammyUSACorporation,           // 0x53
    GameTek,                       // 0x54
    ParkPlace,                     // 0x55
    LJN,                           // 0x56, 0xDB, 0xFF
    Matchbox,                      // 0x57
    MiltonBradleyCompany,          // 0x59
    Mindscape,                     // 0x5A
    Romstar,                       // 0x5B
    NaxatSoft,                     // 0x5C, 0xD6
    Tradewest,                     // 0x5D
    TitusInteractive,              // 0x60
    OceanSoftware,                 // 0x67
    ElectroBrain,                  // 0x6F
    InterplayEntertainment,        // 0x71
    Broderbund,                    // 0x72, 0xAA
    SculpturedSoftware,            // 0x73
    TheSalesCurveLimited,          // 0x75
    THQ,                           // 0x78
    Accolade,                      // 0x79
    TriffixEntertainment,          // 0x7A
    MicroProse,                    // 0x7C
    MisawaEntertainment,           // 0x80
    LOZCG,                         // 0x83
    TokumaShoten,                  // 0x86, 0xC4
    BulletProofSoftware,           // 0x8B
    VicTokaiCorp,                  // 0x8C
    ApeInc,                        // 0x8E
    IMax,                          // 0x8F
    ChunsoftCo,                    // 0x91
    VideoSystem,                   // 0x92
    TsubarayaProductions,          // 0x93
    Varie,                         // 0x95, 0xE3
    Yonezawa,                      // 0x96
    Arc,                           // 0x99
    NihonBussan,                   // 0x9A
    Tecmo,                         // 0x9B
    Imagineer,                     // 0x9C
    Nova,                          // 0x9F
    HoriElectric,                  // 0xA1
    Kawada,                        // 0xA6
    Takara,                        // 0xA7
    TechnosJapan,                  // 0xA9
    ToeiAnimation,                 // 0xAC
    Toho,                          // 0xAD
    Namco,                         // 0xAF
    ASCIICorporation,              // 0xB1
    SquareEnix,                    // 0xB4
    HALLaboratory,                 // 0xB6
    SNK,                           // 0xB7
    PonyCanyon,                    // 0xB9, 0xCE
    CultureBrain,                  // 0xBA
    Sunsoft,                       // 0xBB
    SonyImagesoft,                 // 0xBD
    SammyCorporation,              // 0xBF
    Taito,                         // 0xC0, 0xD0
    Square,                        // 0xC3
    DataEast,                      // 0xC5
    TonkinHouse,                   // 0xC6
    Koei,                          // 0xC8
    UFL,                           // 0xC9
    UltraGames,                    // 0xCA
    VAPInc,                        // 0xCB
    UseCorporation,                // 0xCC
    Meldac,                        // 0xCD
    SOFEL,                         // 0xD1
    Quest,                         // 0xD2
    SigmaEnterprises,              // 0xD3
    ASKKodansha,                   // 0xD4
    CopyaSystem,                   // 0xD7
    Tomy,                          // 0xDA
    NipponComputerSystems,         // 0xDD
    HumanEnt,                      // 0xDE
    Altron,                        // 0xDF
    TowaChiki,                     // 0xE1
    Yutaka,                        // 0xE2
    Epoch,                         // 0xE5
    Athena,                        // 0xE7
    AsmikAceEntertainment,         // 0xE8
    Natsume,                       // 0xE9
    KingRecords,                   // 0xEA
    EpicSonyRecords,               // 0xEC
    IGS,                           // 0xEE
    AWave,                         // 0xF0
    ExtremeEntertainment,          // 0xF3
    Unknown
}

impl OldLicensee {
    /// Convert from a byte value to an OldLicensee enum variant
    pub fn from_byte(value: u8) -> Self {
        match value {
            0x00 => Self::None,
            0x01 | 0x31 => Self::Nintendo,
            0x08 | 0x38 => Self::Capcom,
            0x09 => Self::HOTB,
            0x0A | 0xE0 => Self::Jaleco,
            0x0B => Self::CoconutsJapan,
            0x0C | 0x6E => Self::EliteSystems,
            0x13 | 0x69 => Self::ElectronicArts,
            0x18 => Self::HudsonSoft,
            0x19 => Self::ITCEntertainment,
            0x1A => Self::Yanoman,
            0x1D => Self::JapanClary,
            0x1F | 0x4A | 0x61 => Self::VirginGames,
            0x24 => Self::PCMComplete,
            0x25 => Self::SanX,
            0x28 | 0x7F | 0x97 | 0xC2 => Self::Kemco,
            0x29 => Self::SETACorporation,
            0x30 | 0x70 => Self::Infogrames,
            0x32 | 0xA2 | 0xB2 => Self::Bandai,
            0x33 => Self::UseNewLicenseeCode,
            0x34 | 0xA4 => Self::Konami,
            0x35 => Self::HectorSoft,
            0x39 | 0x9D | 0xD9 => Self::Banpresto,
            0x3C => Self::EntertainmentInteractive,
            0x3E => Self::Gremlin,
            0x41 => Self::UbiSoft,
            0x42 | 0xEB => Self::Atlus,
            0x44 | 0x4D => Self::MalibuInteractive,
            0x46 | 0xCF => Self::Angel,
            0x47 => Self::SpectrumHoloByte,
            0x49 => Self::Irem,
            0x4F => Self::USGold,
            0x50 => Self::Absolute,
            0x51 | 0xB0 => Self::AcclaimEntertainment,
            0x52 => Self::Activision,
            0x53 => Self::SammyUSACorporation,
            0x54 => Self::GameTek,
            0x55 => Self::ParkPlace,
            0x56 | 0xDB | 0xFF => Self::LJN,
            0x57 => Self::Matchbox,
            0x59 => Self::MiltonBradleyCompany,
            0x5A => Self::Mindscape,
            0x5B => Self::Romstar,
            0x5C | 0xD6 => Self::NaxatSoft,
            0x5D => Self::Tradewest,
            0x60 => Self::TitusInteractive,
            0x67 => Self::OceanSoftware,
            0x6F => Self::ElectroBrain,
            0x71 => Self::InterplayEntertainment,
            0x72 | 0xAA => Self::Broderbund,
            0x73 => Self::SculpturedSoftware,
            0x75 => Self::TheSalesCurveLimited,
            0x78 => Self::THQ,
            0x79 => Self::Accolade,
            0x7A => Self::TriffixEntertainment,
            0x7C => Self::MicroProse,
            0x80 => Self::MisawaEntertainment,
            0x83 => Self::LOZCG,
            0x86 | 0xC4 => Self::TokumaShoten,
            0x8B => Self::BulletProofSoftware,
            0x8C => Self::VicTokaiCorp,
            0x8E => Self::ApeInc,
            0x8F => Self::IMax,
            0x91 => Self::ChunsoftCo,
            0x92 => Self::VideoSystem,
            0x93 => Self::TsubarayaProductions,
            0x95 | 0xE3 => Self::Varie,
            0x96 => Self::Yonezawa,
            0x99 => Self::Arc,
            0x9A => Self::NihonBussan,
            0x9B => Self::Tecmo,
            0x9C => Self::Imagineer,
            0x9F => Self::Nova,
            0xA1 => Self::HoriElectric,
            0xA6 => Self::Kawada,
            0xA7 => Self::Takara,
            0xA9 => Self::TechnosJapan,
            0xAC => Self::ToeiAnimation,
            0xAD => Self::Toho,
            0xAF => Self::Namco,
            0xB1 => Self::ASCIICorporation,
            0xB4 => Self::SquareEnix,
            0xB6 => Self::HALLaboratory,
            0xB7 => Self::SNK,
            0xB9 | 0xCE => Self::PonyCanyon,
            0xBA => Self::CultureBrain,
            0xBB => Self::Sunsoft,
            0xBD => Self::SonyImagesoft,
            0xBF => Self::SammyCorporation,
            0xC0 | 0xD0 => Self::Taito,
            0xC3 => Self::Square,
            0xC5 => Self::DataEast,
            0xC6 => Self::TonkinHouse,
            0xC8 => Self::Koei,
            0xC9 => Self::UFL,
            0xCA => Self::UltraGames,
            0xCB => Self::VAPInc,
            0xCC => Self::UseCorporation,
            0xCD => Self::Meldac,
            0xD1 => Self::SOFEL,
            0xD2 => Self::Quest,
            0xD3 => Self::SigmaEnterprises,
            0xD4 => Self::ASKKodansha,
            0xD7 => Self::CopyaSystem,
            0xDA => Self::Tomy,
            0xDD => Self::NipponComputerSystems,
            0xDE => Self::HumanEnt,
            0xDF => Self::Altron,
            0xE1 => Self::TowaChiki,
            0xE2 => Self::Yutaka,
            0xE5 => Self::Epoch,
            0xE7 => Self::Athena,
            0xE8 => Self::AsmikAceEntertainment,
            0xE9 => Self::Natsume,
            0xEA => Self::KingRecords,
            0xEC => Self::EpicSonyRecords,
            0xEE => Self::IGS,
            0xF0 => Self::AWave,
            0xF3 => Self::ExtremeEntertainment,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct CartridgeHeader {
    title: String,
    manufacturer_code: String,
    new_licensee: NewLicensee,
    sgb_flag: bool,
    mbc: mbc::MBC,
    destination: Destination,
    old_licensee: OldLicensee,
    version_number: u8,

}
impl GameBoy {
    pub fn decode_cart_header(&self) -> CartridgeHeader {
        let mbc = self.detect_mbc();
        let title = String::from_utf8(self.memory.cartridge[0x0134..0x0143].to_vec()).unwrap();
        let manufacturer_code = String::from_utf8(self.memory.cartridge[0x013F..=0x0142].to_vec()).unwrap();
        let new_licensee = NewLicensee::from_bytes(&self.memory.cartridge[0x0144..=0x0145].try_into().unwrap());
        let sgb_flag = self.memory.cartridge[0x0147] == 0x03;
        let destination = match self.memory.cartridge[0x014A] {
            0x00 => {Destination::Japan},
            0x01 => {Destination::RestOfWorld},
            _ => {Destination::Undefined},
        };
        let old_licensee = OldLicensee::from_byte(self.memory.cartridge[0x014B]);
        let version_number = self.memory.cartridge[0x014C];

        CartridgeHeader {
            title,
            manufacturer_code,
            new_licensee,
            sgb_flag,
            mbc,
            destination,
            old_licensee,
            version_number,
        }

    }
}