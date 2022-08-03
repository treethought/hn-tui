use cursive::align::HAlign;
use cursive::event::EventResult;
use cursive::theme::{Color, PaletteColor};
use cursive::traits::*;
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, OnEventView, ScrollView, SelectView,
    TextView,
};
use cursive::{Cursive, CursiveExt};
use html_parser::{Dom, Element, Node};

#[derive(Debug, Default)] // debug for printing, Default for initializing empty
struct HNPost {
    title: String,
    link: String,
    id: String,
}

impl HNPost {
    fn default() -> Self {
        Self {
            title: String::new(),
            link: String::new(),
            id: String::new(),
        }
    }
}

// extracts post title and link from the "titlelink" class post element
fn extract_title_link(elem: &Element) -> (String, String) {
    let (mut title, mut link) = (String::new(), String::new());
    if let Some(href) = elem.attributes.get("href") {
        if let Some(s) = href {
            link = s.to_string();
        }
    }
    // get the text node of the element
    for n in elem.children.iter() {
        match n {
            Node::Text(t) => {
                title = String::from(t);
                break;
            }
            _ => continue,
        }
    }
    // dbg!((&title, &link));
    (title, link)
}

fn build_posts_view(posts: Vec<HNPost>) -> SelectView {
    let mut posts_list = SelectView::new().h_align(HAlign::Center).autojump();

    for p in posts.iter() {
        if p.title.is_empty() || p.link.is_empty() {
            continue;
        }
        posts_list.add_item(p.title.clone(), p.link.clone());
    }

    let posts_list = posts_list.on_submit(select_post);

    // let posts_list = OnEventView::new(posts_list)
    //     .on_pre_event_inner('k', |s, _| {
    //         let cb = s.select_up(1);
    //         Some(EventResult::Consumed(Some(cb)))
    //     })
    //     .on_pre_event_inner('j', |s, _| {
    //     let cb = s.select_down(1);
    //         Some(EventResult::Consumed(Some(cb)))

    // })

    posts_list
}

fn select_post(s: &mut Cursive, url: &str) {
    s.pop_layer();
    s.add_layer(Dialog::text(url).button("Quit", Cursive::quit));
}

fn extract_by_class(nodes: &Vec<Node>, class: &str) -> Vec<Node> {
    let mut result: Vec<Node> = Vec::new();
    for n in nodes.iter() {
        match n {
            Node::Element(elem) => {
                for c in elem.classes.iter() {
                    if c.eq(class) {
                        result.push(n.clone());
                    }
                }
                // also need to extract any nodes from this node's children
                result.extend(extract_by_class(&elem.children, class));
            }
            _ => continue,
        }
    }
    result
}

fn extract_hn_posts(nodes: &Vec<Node>) -> Vec<HNPost> {
    let mut posts = Vec::new();
    let post_nodes = extract_by_class(nodes, "athing");

    for n in post_nodes.iter() {
        if let Node::Element(el) = n {
            // init post from link and title
            let mut p = HNPost::default();
            for tn in extract_by_class(&el.children, "titlelink").iter() {
                if let Node::Element(el) = tn {
                    (p.title, p.link) = extract_title_link(el);
                }
            }

            if let Some(id) = el.id.clone() {
                p.id = id
            }
            posts.push(p);
        }
    }

    posts
}

async fn fetch_hn_posts() -> Vec<HNPost> {
    let resp = reqwest::get("https://news.ycombinator.com")
        .await
        .unwrap()
        .text()
        .await;

    let html = resp.expect("failed to get hn posts");

    let dom = Dom::parse(&html[..]).expect("failed to parse dom");

    let posts = extract_hn_posts(&dom.children);

    posts
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut siv = Cursive::default();

    let mut theme = siv.current_theme().clone();
    theme.palette[PaletteColor::Background] = Color::TerminalDefault;
    siv.set_theme(theme);

    let welcome = TextView::new("fetching posts");
    siv.add_layer(welcome);

    let hn_posts = fetch_hn_posts().await;
    // siv.pop_layer();

    let posts = build_posts_view(hn_posts);
    // siv.add_layer(posts);

    let buttons = LinearLayout::vertical()
        .child(DummyView)
        .child(Button::new("Quit", Cursive::quit));

    siv.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(ScrollView::new(posts))
                .child(buttons),
        )
        .title("hntui")
        .full_screen(),
    );
    siv.run();
    Ok(())
}
