use ahash::HashMap;
use std::sync::{Arc, RwLock};
use crate::Args;
pub static GLYPH_WIDTH: i16 = 700;
pub static GLYPH_HEIGHT: i16 = 1500;
pub static MIN_GAP: i16 = 200;
pub static UNDERBAR: u8 = 0b00000001;
pub static UNDERDOT: u8 = 0b00000010;

lazy_static::lazy_static! {
    pub static ref ARGS: Arc<RwLock<Args>> = Arc::new(RwLock::new(Args::default()));
    pub static ref COMPOSITE_CHOSUNGS_TO_MAKE: Vec<Vec<Vec<u16>>> = {
        let mut v: Vec<Vec<Vec<u16>>> = Vec::new();
        v.push(vec![vec![0x3131], vec![0x1100], vec![], vec![]]);
        v.push(vec![vec![0x3132], vec![0x1101], vec![], vec![]]);
        v.push(vec![vec![0x3133], vec![0x1100, 0x1109], vec![], vec![]]);
        v.push(vec![vec![0x3134], vec![0x1102], vec![], vec![]]);
        v.push(vec![vec![0x3135], vec![0x1102, 0x110c], vec![], vec![]]);
        v.push(vec![vec![0x3136], vec![0x1102, 0x1112], vec![], vec![]]);
        v.push(vec![vec![0x3137], vec![0x1103], vec![], vec![]]);
        v.push(vec![vec![0x3138], vec![0x1104], vec![], vec![]]);
        v.push(vec![vec![0x3139], vec![0x1105], vec![], vec![]]);
        v.push(vec![vec![0x313a], vec![0x1105, 0x1100], vec![], vec![]]);
        v.push(vec![vec![0x313b], vec![0x1105, 0x1106], vec![], vec![]]);
        v.push(vec![vec![0x313c], vec![0x1105, 0x1107], vec![], vec![]]);
        v.push(vec![vec![0x313d], vec![0x1105, 0x1109], vec![], vec![]]);
        v.push(vec![vec![0x313e], vec![0x1105, 0x1110], vec![], vec![]]);
        v.push(vec![vec![0x313f], vec![0x1105, 0x1111], vec![], vec![]]);
        v.push(vec![vec![0x3140], vec![0x1105, 0x1112], vec![], vec![]]);
        v.push(vec![vec![0x3141], vec![0x1106], vec![], vec![]]);
        v.push(vec![vec![0x3142], vec![0x1107], vec![], vec![]]);
        v.push(vec![vec![0x3143], vec![0x1108], vec![], vec![]]);
        v.push(vec![vec![0x3144], vec![0x1107, 0x1109], vec![], vec![]]);
        v.push(vec![vec![0x3145], vec![0x1109], vec![], vec![]]);
        v.push(vec![vec![0x3146], vec![0x110a], vec![], vec![]]);
        v.push(vec![vec![0x3147], vec![0x110b], vec![], vec![]]);
        v.push(vec![vec![0x3148], vec![0x110c], vec![], vec![]]);
        v.push(vec![vec![0x3149], vec![0x110d], vec![], vec![]]);
        v.push(vec![vec![0x314a], vec![0x110e], vec![], vec![]]);
        v.push(vec![vec![0x314b], vec![0x110f], vec![], vec![]]);
        v.push(vec![vec![0x314c], vec![0x1110], vec![], vec![]]);
        v.push(vec![vec![0x314d], vec![0x1111], vec![], vec![]]);
        v.push(vec![vec![0x314e], vec![0x1112], vec![], vec![]]);
        v.push(vec![vec![0x314f], vec![0x1161], vec![], vec![]]);
        v.push(vec![vec![0x3150], vec![0x1162], vec![], vec![]]);
        v.push(vec![vec![0x3151], vec![0x1163], vec![], vec![]]);
        v.push(vec![vec![0x3152], vec![0x1164], vec![], vec![]]);
        v.push(vec![vec![0x3153], vec![0x1165], vec![], vec![]]);
        v.push(vec![vec![0x3154], vec![0x1166], vec![], vec![]]);
        v.push(vec![vec![0x3155], vec![0x1167], vec![], vec![]]);
        v.push(vec![vec![0x3156], vec![0x1168], vec![], vec![]]);
        v.push(vec![vec![0x3157], vec![0x1169], vec![], vec![]]);
        v.push(vec![vec![0x3158], vec![0x1169, 0x1161], vec![], vec![]]);
        v.push(vec![vec![0x3159], vec![0x1169, 0x1162], vec![], vec![]]);
        v.push(vec![vec![0x315a], vec![0x1169, 0x1175], vec![], vec![]]);
        v.push(vec![vec![0x315b], vec![0x116d], vec![], vec![]]);
        v.push(vec![vec![0x315c], vec![0x116e], vec![], vec![]]);
        v.push(vec![vec![0x315d], vec![0x116e, 0x1165], vec![], vec![]]);
        v.push(vec![vec![0x315e], vec![0x116e, 0x1166], vec![], vec![]]);
        v.push(vec![vec![0x315f], vec![0x116e, 0x1175], vec![], vec![]]);
        v.push(vec![vec![0x3160], vec![0x1172], vec![], vec![]]);
        v.push(vec![vec![0x3161], vec![0x1173], vec![], vec![]]);
        v.push(vec![vec![0x3162], vec![0x1173, 0x1175], vec![], vec![]]);
        v.push(vec![vec![0x3163], vec![0x1175], vec![], vec![]]);
        v.push(vec![vec![0x3165], vec![0x1102, 0x1102], vec![], vec![]]);
        v.push(vec![vec![0x3166], vec![0x1102, 0x1103], vec![], vec![]]);
        v.push(vec![vec![0x318d], vec![0x119e], vec![], vec![]]);
        v
    };
    pub static ref CHO_CONVERSION_TABLE: HashMap<u16, Vec<u16>> = {
        let mut m = HashMap::default();
        m.insert(0x1113, vec![0x1102, 0x1100]);
        m.insert(0x1114, vec![0x1102, 0x1102]);
        m.insert(0x1115, vec![0x1102, 0x1103]);
        m.insert(0x1116, vec![0x1102, 0x1107]);
        m.insert(0x1117, vec![0x1103, 0x1100]);
        m.insert(0x1118, vec![0x1105, 0x1102]);
        m.insert(0x1119, vec![0x1105, 0x1105]);
        m.insert(0x111a, vec![0x1105, 0x1112]);
        m
    };
    pub static ref JUNG_CONVERSION_TABLE: HashMap<u16, Vec<u16>> = {
        let mut m = HashMap::default();
        m.insert(0x116a, vec![0x1169, 0x1161]);
        m.insert(0x116b, vec![0x1169, 0x1162]);
        m.insert(0x116c, vec![0x1169, 0x1175]);
        m.insert(0x116f, vec![0x116e, 0x1165]);
        m.insert(0x1170, vec![0x116e, 0x1166]);
        m.insert(0x1171, vec![0x116e, 0x1175]);
        m.insert(0x1174, vec![0x1173, 0x1175]);
        m
    };
    pub static ref JONG_CONVERSION_TABLE: HashMap<u16, Vec<u16>> = {
        let mut m = HashMap::default();
        m.insert(0x11aa, vec![0x11a8, 0x11ba]);
        m.insert(0x11ac, vec![0x11ab, 0x11ba]);
        m.insert(0x11ad, vec![0x11ab, 0x11ba]);
        m.insert(0x11b0, vec![0x11af, 0x11ba]);
        m.insert(0x11b1, vec![0x11af, 0x11ba]);
        m.insert(0x11b2, vec![0x11af, 0x11ba]);
        m.insert(0x11b3, vec![0x11af, 0x11ba]);
        m.insert(0x11b4, vec![0x11af, 0x11ba]);
        m.insert(0x11b5, vec![0x11af, 0x11ba]);
        m.insert(0x11b6, vec![0x11af, 0x11ba]);
        m.insert(0x11b9, vec![0x11b8, 0x11ba]);
        m.insert(0x11a7, vec![]);
        m
    };
}

