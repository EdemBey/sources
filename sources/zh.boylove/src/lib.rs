#![no_std]

mod html;
mod json;
mod net;
mod setting;

use aidoku::{
	Chapter, DeepLinkHandler, DeepLinkResult, DynamicFilters, Filter, FilterValue, HashMap, Home,
	HomeLayout, Listing, ListingProvider, Manga, MangaPageResult, NotificationHandler, Page,
	Result, Source, WebLoginHandler,
	alloc::{String, Vec},
	bail, error,
	imports::std::send_partial_result,
	register_source,
};
use html::{
	ChapterPage as _, FiltersPage as _, HomePage as _, MangaPage as _, TryElement as _,
	TrySelector as _,
};
use json::{daily_update, manga_page_result, random};
use net::{Api, Url};
use setting::change_charset;

struct Boylove;

impl Source for Boylove {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		let manga_page_result = Url::from_query_or_filters(query.as_deref(), page, &filters)?
			.request()?
			.json_owned::<manga_page_result::Root>()?
			.into();
		Ok(manga_page_result)
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		let manga_page = Url::manga(&manga.key).request()?.html()?;

		if needs_details {
			let updated_details = manga_page.manga_details()?;

			manga = Manga {
				chapters: manga.chapters,
				..updated_details
			};

			if needs_chapters {
				send_partial_result(&manga);
			} else {
				return Ok(manga);
			}
		}

		manga.chapters = manga_page.chapters()?;

		Ok(manga)
	}

	fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		Api::chapter(&chapter.key).request()?.html()?.pages()
	}
}

impl DeepLinkHandler for Boylove {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		let mut splits = url.split('/').skip(3);
		let deep_link_result = match (
			splits.next(),
			splits.next(),
			splits.next(),
			splits.next(),
			splits.next(),
		) {
			(Some("home"), Some("book"), Some("index"), Some("id"), Some(key)) => {
				Some(DeepLinkResult::Manga { key: key.into() })
			}

			(Some("home"), Some("book"), Some("capter"), Some("id"), Some(key)) => {
				let path = Url::chapter(key)
					.request()?
					.html()?
					.try_select_first("a.back")?
					.try_attr("href")?;
				let manga_key = path
					.rsplit_once('/')
					.ok_or_else(|| error!("Character not found: `/`"))?
					.1;

				Some(DeepLinkResult::Chapter {
					manga_key: manga_key.into(),
					key: key.into(),
				})
			}

			(Some("home"), Some("index"), Some("dailyupdate1"), None, None) => {
				let id = Url::DailyUpdatePage
					.request()?
					.html()?
					.try_select_first("ul.stui-list > li.active")?
					.text()
					.ok_or_else(|| {
						error!("No text content for selector: `ul.stui-list > li.active`",)
					})?;

				Some(DeepLinkResult::Listing(Listing {
					id: id.clone(),
					name: id,
					..Default::default()
				}))
			}

			(
				Some("home"),
				Some("index"),
				Some("dailyupdate1"),
				Some("weekday"),
				Some(week_of_day),
			) => {
				let id = match week_of_day {
					"11" => "最新",
					"6" => "週日",
					"0" => "週一",
					"1" => "週二",
					"2" => "週三",
					"3" => "週四",
					"4" => "週五",
					"5" => "週六",
					_ => return Ok(None),
				};

				Some(DeepLinkResult::Listing(Listing {
					id: id.into(),
					name: id.into(),
					..Default::default()
				}))
			}

			(
				Some("home"),
				Some("index"),
				Some("pages"),
				Some("w"),
				Some("recommend.html" | "recommend"),
			) => Some(DeepLinkResult::Listing(Listing {
				id: "無碼專區".into(),
				name: "無碼專區".into(),
				..Default::default()
			})),

			(
				Some("home"),
				Some("index"),
				Some("pages"),
				Some("w"),
				Some("topestmh.html" | "topestmh"),
			) => Some(DeepLinkResult::Listing(Listing {
				id: "排行榜".into(),
				name: "排行榜".into(),
				..Default::default()
			})),

			_ => None,
		};
		Ok(deep_link_result)
	}
}

impl DynamicFilters for Boylove {
	fn get_dynamic_filters(&self) -> Result<Vec<Filter>> {
		let tags = Url::FiltersPage.request()?.html()?.tags_filter()?;

		let filters = [tags].into();
		Ok(filters)
	}
}

impl Home for Boylove {
	fn get_home(&self) -> Result<HomeLayout> {
		Url::Home.request()?.html()?.home_layout()
	}
}

impl ListingProvider for Boylove {
	fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		let manga_page_result = match listing.id.as_str() {
			id @ ("最新" | "週日" | "週一" | "週二" | "週三" | "週四" | "週五" | "週六") => {
				Url::daily_update(id, page)?
					.request()?
					.json_owned::<daily_update::Root>()?
					.into()
			}

			id @ ("無碼專區" | "排行榜") => Url::listing(id, page)?
				.request()?
				.json_owned::<manga_page_result::Root>()?
				.into(),

			"猜你喜歡" => Url::random()
				.request()?
				.json_owned::<random::Root>()?
				.into(),

			id => bail!("Invalid listing ID: `{id}`"),
		};
		Ok(manga_page_result)
	}
}

impl NotificationHandler for Boylove {
	fn handle_notification(&self, notification: String) {
		if notification == "updatedCharset" {
			_ = change_charset();
		}
	}
}

impl WebLoginHandler for Boylove {
	fn handle_web_login(&self, key: String, cookies: HashMap<String, String>) -> Result<bool> {
		if key != "login" {
			bail!("Invalid login key: `{key}`");
		}

		let is_logged_in = cookies.get("rfv").is_some();
		Ok(is_logged_in)
	}
}

register_source!(
	Boylove,
	DeepLinkHandler,
	DynamicFilters,
	Home,
	ListingProvider,
	NotificationHandler,
	WebLoginHandler
);
