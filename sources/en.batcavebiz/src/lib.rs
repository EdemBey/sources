#![no_std]
use aidoku::{
	alloc::{vec, String, Vec},
	imports::{html::Document, net::Request},
	prelude::*,
	HomeComponent, HomeLayout, Link, LinkValue, Listing, ListingKind, Manga, MangaPageResult,
	Result, Source,
};
use gigaviewer::{GigaViewer, Impl, Params};

const BASE_URL: &str = "https://batcave.biz";

struct BatCaveBiz;

impl Impl for BatCaveBiz {
	fn new() -> Self {
		Self
	}

	fn params(&self) -> Params {
		Params {
			base_url: BASE_URL.into(),
			..Default::default()
		}
	}

	fn get_manga_list(&self, _params: &Params, _listing: Listing, _page: i32) -> Result<MangaPageResult> {
		let html = Request::get(format!("{}/comix/", BASE_URL))?.html()?;
		let mut entries: Vec<Manga> = Vec::new();

		for node in html.select("div.shortstory") {
			let a = node.select_first("a")?;
			let title = a.attr("title").unwrap_or("").to_string();
			let url = a.attr("href").unwrap_or("").strip_prefix(BASE_URL).unwrap_or("").to_string();
			let thumb = a.select_first("img").and_then(|img| img.attr("src")).unwrap_or("").to_string();

			entries.push(Manga {
				title,
				cover: Some(thumb),
				key: url,
				..Default::default()
			});
		}

		Ok(MangaPageResult {
			entries,
			has_next_page: false,
		})
	}

	fn get_home(&self, _params: &Params) -> Result<HomeLayout> {
		let html = Request::get(format!("{}/comix/", BASE_URL))?.html()?;
		let mut entries: Vec<Link> = Vec::new();

		for node in html.select("div.shortstory") {
			let a = node.select_first("a")?;
			let title = a.attr("title").unwrap_or("").to_string();
			let url = a.attr("href").unwrap_or("").strip_prefix(BASE_URL).unwrap_or("").to_string();
			let thumb = a.select_first("img").and_then(|img| img.attr("src")).unwrap_or("").to_string();

			let manga = Manga {
				title: title.clone(),
				cover: Some(thumb.clone()),
				key: url.clone(),
				..Default::default()
			};

			entries.push(Link {
				title,
				image_url: Some(thumb),
				value: Some(LinkValue::Manga(manga)),
				..Default::default()
			});
		}

		Ok(HomeLayout {
			components: vec![
				HomeComponent {
					title: Some("Каталог".into()),
					subtitle: None,
					value: aidoku::HomeComponentValue::Scroller {
						entries,
						listing: None,
					},
				}
			]
		})
	}
}

register_source!(
	GigaViewer<BatCaveBiz>,
	PageImageProcessor,
	Home,
	BasicLoginHandler,
	NotificationHandler,
	DeepLinkHandler
);
