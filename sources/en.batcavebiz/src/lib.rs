#![no_std]
use aidoku::{prelude::*, Source};
use mangabox::{Impl, MangaBox, Params};

const BASE_URL: &str = "https://batcave.biz";

struct BatCave;

impl Impl for BatCave {
	fn new() -> Self {
		Self
	}

	fn params(&self) -> Params {
		Params {
			base_url: BASE_URL.into(),
			..Default::default()
		}
	}
}

register_source!(
	MangaBox<BatCave>,
	ListingProvider,
	Home,
	ImageRequestProvider,
	DeepLinkHandler
);
