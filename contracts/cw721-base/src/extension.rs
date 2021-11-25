use cosmwasm_std::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct PlayibleInfo {
    /// Reference ID of the Athlete Token
    pub athlete_id: String,
    /// Describes the rarity of the NFT 
    pub rarity: String,
    /// Determines whether or not the NFT is locked for Fantasy Sports
    pub is_locked: bool,
    /// Determines the unlock date after the NFT has been locked
    pub unlock_date: Option<Timestamp>,
    /// Number of times an NFt can be locked up for a game
    pub usage: u64,
}

pub trait PlayiblePersonalization {
    fn get_athlete_id(&self) -> String;
    fn set_athlete_id(&mut self, athlete_id: &str);

    fn get_rarity(&self) -> String;
    fn set_rarity(&mut self, rarity: &str);

    fn get_is_locked(&self) -> bool;
    fn set_is_locked(&mut self, is_locked: bool);

    fn get_unlock_date(&self) -> Option<Timestamp>;
    fn set_unlock_date(&mut self, unlock_date: Option<Timestamp>);

    fn get_usage(&self) -> u64;
    fn set_usage(&mut self, usage: u64);
}

impl PlayiblePersonalization for PlayibleInfo {
    fn get_athlete_id(&self) -> String {
        self.athlete_id.clone()
    }
    fn set_athlete_id(&mut self, athlete_id: &str) {
        self.athlete_id = String::from(athlete_id)
    }

    fn get_rarity(&self) -> String {
        self.rarity.clone()
    }
    fn set_rarity(&mut self, rarity: &str) {
        self.rarity = String::from(rarity)
    }

    fn get_is_locked(&self) -> bool {
        self.is_locked.clone()
    }
    fn set_is_locked(&mut self, is_locked: bool) {
        self.is_locked = is_locked
    }

    fn get_unlock_date(&self) -> Option<Timestamp> {
        self.unlock_date.clone()
    }
    fn set_unlock_date(&mut self, unlock_date: Option<Timestamp>) {
        self.unlock_date = unlock_date
    }

    fn get_usage(&self) -> u64 {
        self.usage.clone()
    }
    fn set_usage(&mut self, usage: u64) {
        self.usage = usage
    }
}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

pub trait MetaDataPersonalization {
    fn get_decision_trait(&self, trait_type: &str) -> Option<Trait>;
    fn set_personalized_trait(&mut self, trait_type: &str, value: &str);
    fn set_status(&mut self, status: &str);
    fn get_status(&self) -> Option<String>;
    fn get_token_uri(&self) -> String;
    fn get_image(&self, prefix: &str) -> Option<String>;
    fn set_image(&mut self, image: Option<String>);
}

pub trait MetaPersonalize {
    fn perform_mint(&self, mint_meta: &mut dyn MetaDataPersonalization) -> Option<String>;
}
// see: https://docs.opensea.io/docs/metadata-standards
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Metadata {
    pub token_uri: String,
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    pub current_status: Option<String>,
}

impl MetaDataPersonalization for Metadata {
    fn get_decision_trait(&self, trait_type: &str) -> Option<Trait> {
        if let Some(attr) = &self.attributes {
            let trait_attribute = attr
                .iter()
                .filter(|t| t.trait_type == trait_type)
                .cloned()
                .collect::<Vec<_>>();
            trait_attribute.first().cloned()
        } else {
            None
        }
    }
    fn get_token_uri(&self) -> String {
        self.token_uri.clone()
    }

    fn set_personalized_trait(&mut self, trait_type: &str, value: &str) {
        if let Some(attr_list) = &self.attributes {
            let mut new_attr: Vec<Trait> = Default::default();
            for att in attr_list {
                if att.trait_type == trait_type {
                    new_attr.push(Trait {
                        display_type: att.display_type.clone(),
                        trait_type: att.trait_type.clone(),
                        value: value.into(),
                    })
                } else {
                    new_attr.push(att.clone());
                }
            }
            self.name = Some(String::from(value));
            //  self.attributes = Some(new_attr);
            self.attributes = Some(new_attr);
        }
    }
    fn set_status(&mut self, status: &str) {
        self.current_status = Some(String::from(status))
    }
    fn get_status(&self) -> Option<String> {
        self.current_status.clone()
    }
    fn get_image(&self, prefix: &str) -> Option<String> {
        return self.image.as_ref().map(|i| {
            if i.starts_with("ipfs://") || i.starts_with("http") {
                i.clone()
            } else {
                format!("{}{}", prefix, i)
            }
        });
    }
    fn set_image(&mut self, image: Option<String>) {
        self.image = image
    }
}