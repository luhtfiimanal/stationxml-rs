//! Builder pattern API for constructing inventories.
//!
//! Provides a fluent, closure-based API for building an [`Inventory`]
//! without manually constructing every nested struct.
//!
//! # Example
//!
//! ```
//! use stationxml_rs::Inventory;
//!
//! let inv = Inventory::builder()
//!     .source("Pena Bumi")
//!     .network("XX", |net| {
//!         net.description("Local Test Network")
//!            .station("PBUMI", |sta| {
//!                sta.latitude(-7.7714)
//!                   .longitude(110.3776)
//!                   .elevation(150.0)
//!                   .site_name("Yogyakarta")
//!                   .channel("SHZ", "00", |ch| {
//!                       ch.azimuth(0.0).dip(-90.0).sample_rate(100.0)
//!                   })
//!            })
//!     })
//!     .build();
//!
//! assert_eq!(inv.networks[0].stations[0].channels[0].code, "SHZ");
//! ```

use chrono::{DateTime, Utc};

use crate::inventory::*;

// ─── InventoryBuilder ───────────────────────────────────────────────

/// Builder for [`Inventory`].
pub struct InventoryBuilder {
    source: String,
    sender: Option<String>,
    created: Option<DateTime<Utc>>,
    networks: Vec<Network>,
}

impl Inventory {
    /// Create a new inventory builder.
    pub fn builder() -> InventoryBuilder {
        InventoryBuilder {
            source: String::new(),
            sender: None,
            created: None,
            networks: vec![],
        }
    }
}

impl InventoryBuilder {
    /// Set the source organization.
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Set the sender identifier.
    pub fn sender(mut self, sender: impl Into<String>) -> Self {
        self.sender = Some(sender.into());
        self
    }

    /// Set the creation timestamp.
    pub fn created(mut self, created: DateTime<Utc>) -> Self {
        self.created = Some(created);
        self
    }

    /// Add a network using a closure-based builder.
    pub fn network(
        mut self,
        code: impl Into<String>,
        f: impl FnOnce(NetworkBuilder) -> NetworkBuilder,
    ) -> Self {
        let builder = f(NetworkBuilder::new(code));
        self.networks.push(builder.build());
        self
    }

    /// Build the final [`Inventory`].
    pub fn build(self) -> Inventory {
        Inventory {
            source: self.source,
            sender: self.sender,
            created: self.created,
            networks: self.networks,
        }
    }
}

// ─── NetworkBuilder ─────────────────────────────────────────────────

/// Builder for [`Network`].
pub struct NetworkBuilder {
    code: String,
    description: Option<String>,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
    stations: Vec<Station>,
}

impl NetworkBuilder {
    fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            description: None,
            start_date: None,
            end_date: None,
            stations: vec![],
        }
    }

    /// Set the network description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the start date.
    pub fn start_date(mut self, date: DateTime<Utc>) -> Self {
        self.start_date = Some(date);
        self
    }

    /// Set the end date.
    pub fn end_date(mut self, date: DateTime<Utc>) -> Self {
        self.end_date = Some(date);
        self
    }

    /// Add a station using a closure-based builder.
    pub fn station(
        mut self,
        code: impl Into<String>,
        f: impl FnOnce(StationBuilder) -> StationBuilder,
    ) -> Self {
        let builder = f(StationBuilder::new(code));
        self.stations.push(builder.build());
        self
    }

    fn build(self) -> Network {
        Network {
            code: self.code,
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            stations: self.stations,
        }
    }
}

// ─── StationBuilder ─────────────────────────────────────────────────

/// Builder for [`Station`].
pub struct StationBuilder {
    code: String,
    description: Option<String>,
    latitude: f64,
    longitude: f64,
    elevation: f64,
    site_name: String,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
    creation_date: Option<DateTime<Utc>>,
    channels: Vec<Channel>,
}

impl StationBuilder {
    fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            description: None,
            latitude: 0.0,
            longitude: 0.0,
            elevation: 0.0,
            site_name: String::new(),
            start_date: None,
            end_date: None,
            creation_date: None,
            channels: vec![],
        }
    }

    pub fn latitude(mut self, lat: f64) -> Self {
        self.latitude = lat;
        self
    }

    pub fn longitude(mut self, lon: f64) -> Self {
        self.longitude = lon;
        self
    }

    pub fn elevation(mut self, elev: f64) -> Self {
        self.elevation = elev;
        self
    }

    pub fn site_name(mut self, name: impl Into<String>) -> Self {
        self.site_name = name.into();
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn start_date(mut self, date: DateTime<Utc>) -> Self {
        self.start_date = Some(date);
        self
    }

    pub fn end_date(mut self, date: DateTime<Utc>) -> Self {
        self.end_date = Some(date);
        self
    }

    /// Add a channel using a closure-based builder.
    ///
    /// Channel lat/lon/elevation default to the station's values if not set.
    pub fn channel(
        mut self,
        code: impl Into<String>,
        location_code: impl Into<String>,
        f: impl FnOnce(ChannelBuilder) -> ChannelBuilder,
    ) -> Self {
        let builder = f(ChannelBuilder::new(
            code,
            location_code,
            self.latitude,
            self.longitude,
            self.elevation,
        ));
        self.channels.push(builder.build());
        self
    }

    fn build(self) -> Station {
        Station {
            code: self.code,
            description: self.description,
            latitude: self.latitude,
            longitude: self.longitude,
            elevation: self.elevation,
            site: Site {
                name: self.site_name,
                ..Default::default()
            },
            start_date: self.start_date,
            end_date: self.end_date,
            creation_date: self.creation_date,
            channels: self.channels,
        }
    }
}

// ─── ChannelBuilder ─────────────────────────────────────────────────

/// Builder for [`Channel`].
pub struct ChannelBuilder {
    code: String,
    location_code: String,
    latitude: f64,
    longitude: f64,
    elevation: f64,
    depth: f64,
    azimuth: f64,
    dip: f64,
    sample_rate: f64,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
    sensor: Option<Equipment>,
    data_logger: Option<Equipment>,
    response: Option<Response>,
}

impl ChannelBuilder {
    fn new(
        code: impl Into<String>,
        location_code: impl Into<String>,
        station_lat: f64,
        station_lon: f64,
        station_elev: f64,
    ) -> Self {
        Self {
            code: code.into(),
            location_code: location_code.into(),
            latitude: station_lat,
            longitude: station_lon,
            elevation: station_elev,
            depth: 0.0,
            azimuth: 0.0,
            dip: 0.0,
            sample_rate: 0.0,
            start_date: None,
            end_date: None,
            sensor: None,
            data_logger: None,
            response: None,
        }
    }

    pub fn latitude(mut self, lat: f64) -> Self {
        self.latitude = lat;
        self
    }

    pub fn longitude(mut self, lon: f64) -> Self {
        self.longitude = lon;
        self
    }

    pub fn elevation(mut self, elev: f64) -> Self {
        self.elevation = elev;
        self
    }

    pub fn depth(mut self, depth: f64) -> Self {
        self.depth = depth;
        self
    }

    pub fn azimuth(mut self, azimuth: f64) -> Self {
        self.azimuth = azimuth;
        self
    }

    pub fn dip(mut self, dip: f64) -> Self {
        self.dip = dip;
        self
    }

    pub fn sample_rate(mut self, rate: f64) -> Self {
        self.sample_rate = rate;
        self
    }

    pub fn start_date(mut self, date: DateTime<Utc>) -> Self {
        self.start_date = Some(date);
        self
    }

    pub fn end_date(mut self, date: DateTime<Utc>) -> Self {
        self.end_date = Some(date);
        self
    }

    pub fn sensor(mut self, sensor: Equipment) -> Self {
        self.sensor = Some(sensor);
        self
    }

    pub fn data_logger(mut self, dl: Equipment) -> Self {
        self.data_logger = Some(dl);
        self
    }

    pub fn response(mut self, response: Response) -> Self {
        self.response = Some(response);
        self
    }

    fn build(self) -> Channel {
        Channel {
            code: self.code,
            location_code: self.location_code,
            latitude: self.latitude,
            longitude: self.longitude,
            elevation: self.elevation,
            depth: self.depth,
            azimuth: self.azimuth,
            dip: self.dip,
            sample_rate: self.sample_rate,
            start_date: self.start_date,
            end_date: self.end_date,
            sensor: self.sensor,
            data_logger: self.data_logger,
            response: self.response,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_minimal() {
        let inv = Inventory::builder().source("Test").build();
        assert_eq!(inv.source, "Test");
        assert!(inv.networks.is_empty());
    }

    #[test]
    fn builder_full() {
        let inv = Inventory::builder()
            .source("Pena Bumi")
            .sender("stationxml-rs")
            .network("XX", |net| {
                net.description("Local Test Network")
                    .station("PBUMI", |sta| {
                        sta.latitude(-7.7714)
                            .longitude(110.3776)
                            .elevation(150.0)
                            .site_name("Yogyakarta Seismic Shelter")
                            .channel("SHZ", "00", |ch| {
                                ch.azimuth(0.0).dip(-90.0).sample_rate(100.0)
                            })
                            .channel("SHN", "00", |ch| {
                                ch.azimuth(0.0).dip(0.0).sample_rate(100.0)
                            })
                            .channel("SHE", "00", |ch| {
                                ch.azimuth(90.0).dip(0.0).sample_rate(100.0)
                            })
                    })
            })
            .build();

        assert_eq!(inv.source, "Pena Bumi");
        assert_eq!(inv.networks.len(), 1);
        assert_eq!(inv.networks[0].code, "XX");

        let sta = &inv.networks[0].stations[0];
        assert_eq!(sta.code, "PBUMI");
        assert_eq!(sta.channels.len(), 3);
        assert_eq!(sta.site.name, "Yogyakarta Seismic Shelter");

        // Channel inherits station coordinates
        let shz = &sta.channels[0];
        assert_eq!(shz.code, "SHZ");
        assert_eq!(shz.latitude, sta.latitude);
        assert_eq!(shz.longitude, sta.longitude);
        assert_eq!(shz.dip, -90.0);

        let she = &sta.channels[2];
        assert_eq!(she.code, "SHE");
        assert_eq!(she.azimuth, 90.0);
        assert_eq!(she.dip, 0.0);
    }

    #[test]
    fn builder_with_sensor() {
        let inv = Inventory::builder()
            .source("Test")
            .network("XX", |net| {
                net.station("TEST", |sta| {
                    sta.latitude(0.0)
                        .longitude(0.0)
                        .elevation(0.0)
                        .site_name("Test")
                        .channel("SHZ", "00", |ch| {
                            ch.azimuth(0.0)
                                .dip(-90.0)
                                .sample_rate(100.0)
                                .sensor(Equipment {
                                    equipment_type: Some("Geophone".into()),
                                    model: Some("GS-11D".into()),
                                    manufacturer: Some("Geospace".into()),
                                    ..Default::default()
                                })
                        })
                })
            })
            .build();

        let sensor = inv.networks[0].stations[0].channels[0]
            .sensor
            .as_ref()
            .unwrap();
        assert_eq!(sensor.model.as_deref(), Some("GS-11D"));
    }
}
