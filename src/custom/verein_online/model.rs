use serde::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct UserId(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct ArtikelId(pub i32);

// #[serde_as]
// #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
// pub struct Member {
//     #[serde_as(as = "DisplayFromStr")]
// 	pub id             : i32,
// 	pub name           : UserName,
// 	pub fotourl        : String,
// 	pub mandatsreferenz: String
// }

#[derive(Debug, Serialize)]
pub struct Bill {
    pub user_id         : UserId,       // id nachschlagen in DataUser.csv. Wenn Spalte 3 ("toBeUsed") == 0 dann skip. Wenn nicht vorhanden dann fallback zum Namen
    pub quelle          : &'static str, // "allgemeiner Beleg"
    pub brutto_netto    : i32,          // 2
    pub artikel_id      : ArtikelId,    // DataMachine.csv#2
    pub positionsdetails: String,       // Date
    pub anzahl          : i32,          // minutes divided by DataMachine.csv#5 (ceil)
    pub rechnungstyp    : i32           // 0
}