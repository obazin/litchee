//! Integration tests for the Users API.

use litchee::LichessClient;
use litchee::api::users::players::UserQuery;
use wiremock::matchers::{body_string_contains, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn get_returns_extended_user() {
    let server = MockServer::start().await;
    let body = r#"{"id":"bobby","username":"Bobby","url":"https://lichess.org/@/Bobby"}"#;
    Mock::given(method("GET"))
        .and(path("/api/user/bobby"))
        .and(query_param("trophies", "true"))
        .and(query_param("fideId", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let user = client(&server)
        .users()
        .get("bobby", &UserQuery::default().trophies(true).fide_id(true))
        .await
        .unwrap();

    assert_eq!(user.user.username, "Bobby");
}

#[tokio::test]
async fn get_many_posts_comma_separated_ids() {
    let server = MockServer::start().await;
    let body = r#"[{"id":"a","username":"A"},{"id":"b","username":"B"}]"#;
    Mock::given(method("POST"))
        .and(path("/api/users"))
        .and(query_param("profile", "true"))
        .and(body_string_contains("a,b"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let users = client(&server)
        .users()
        .get_many(&["a", "b"], Some(true), None)
        .await
        .unwrap();

    assert_eq!(users.len(), 2);
    assert_eq!(users[1].id, "b");
}

#[tokio::test]
async fn statuses_queries_the_ids() {
    let server = MockServer::start().await;
    let body = r#"[{"id":"bobby","name":"Bobby","online":true}]"#;
    Mock::given(method("GET"))
        .and(path("/api/users/status"))
        .and(query_param("ids", "bobby,mary"))
        .and(query_param("withSignal", "true"))
        .and(query_param("withGameIds", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let statuses = client(&server)
        .users()
        .statuses(&["bobby", "mary"], Some(true), Some(true), None)
        .await
        .unwrap();

    assert_eq!(statuses[0].online, Some(true));
}

#[tokio::test]
async fn crosstable_returns_scores() {
    let server = MockServer::start().await;
    let body = r#"{"users":{"a":753.5,"b":459.5},"nbGames":1213}"#;
    Mock::given(method("GET"))
        .and(path("/api/crosstable/a/b"))
        .and(query_param("matchup", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let crosstable = client(&server)
        .users()
        .crosstable("a", "b", true)
        .await
        .unwrap();

    assert_eq!(crosstable.nb_games, 1213);
}

#[tokio::test]
async fn autocomplete_returns_usernames() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/player/autocomplete"))
        .and(query_param("term", "bob"))
        .and(query_param("team", "coders"))
        .and(query_param("friend", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"["bobby","bobbyfischer"]"#))
        .mount(&server)
        .await;

    let names = client(&server)
        .users()
        .autocomplete("bob")
        .team("coders")
        .friend(true)
        .send()
        .await
        .unwrap();

    assert_eq!(names, vec!["bobby", "bobbyfischer"]);
}

#[tokio::test]
async fn autocomplete_exists_returns_bool() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/player/autocomplete"))
        .and(query_param("term", "bobby"))
        .and(query_param("exists", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string("true"))
        .mount(&server)
        .await;

    let exists = client(&server)
        .users()
        .autocomplete("bobby")
        .exists()
        .await
        .unwrap();

    assert!(exists);
}

#[tokio::test]
async fn autocomplete_objects_returns_users() {
    let server = MockServer::start().await;
    let body = r#"{"result":[{"id":"bobby","name":"Bobby","online":true}]}"#;
    Mock::given(method("GET"))
        .and(path("/api/player/autocomplete"))
        .and(query_param("term", "bob"))
        .and(query_param("object", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let users = client(&server)
        .users()
        .autocomplete("bob")
        .objects()
        .await
        .unwrap();

    assert_eq!(users[0].user.name, "Bobby");
    assert_eq!(users[0].online, Some(true));
}

#[tokio::test]
async fn rating_history_returns_entries() {
    let server = MockServer::start().await;
    let body = r#"[{"name":"Bullet","points":[[2011,0,8,1472]]}]"#;
    Mock::given(method("GET"))
        .and(path("/api/user/bobby/rating-history"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let history = client(&server)
        .users()
        .rating_history("bobby")
        .await
        .unwrap();
    assert_eq!(history[0].name, "Bullet");
}

#[tokio::test]
async fn top_uses_perf_and_count_path() {
    let server = MockServer::start().await;
    let body =
        r#"{"users":[{"id":"a","username":"A","perfs":{"bullet":{"rating":2900,"progress":1}}}]}"#;
    Mock::given(method("GET"))
        .and(path("/api/player/top/10/bullet"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let board = client(&server).users().top("bullet", 10).await.unwrap();
    assert_eq!(board.users[0].username, "A");
}

#[tokio::test]
async fn live_streamers_returns_list() {
    let server = MockServer::start().await;
    let body = r#"[{"id":"a","name":"A","stream":{"service":"twitch"}}]"#;
    Mock::given(method("GET"))
        .and(path("/api/streamer/live"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let streamers = client(&server).users().live_streamers().await.unwrap();
    assert_eq!(streamers[0].user.id, "a");
}

#[tokio::test]
async fn write_note_posts_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/user/bobby/note"))
        .and(body_string_contains("text=hello"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;
    client(&server)
        .users()
        .write_note("bobby", "hello")
        .await
        .unwrap();
}

#[tokio::test]
async fn perf_stats_returns_stats() {
    let server = MockServer::start().await;
    let body = r#"{"rank":42,"percentile":98.5,"perf":{"nb":1000,"progress":12},
        "stat":{
            "highest":{"int":2100,"at":"2023-01-01T00:00:00Z","gameId":"g1"},
            "count":{"all":1000,"win":600,"loss":300,"draw":100,"opAvg":1850.5,"seconds":123456},
            "bestWins":{"results":[{"opRating":2200,"opId":{"id":"x","name":"X"},
                "at":"2023-02-01T00:00:00Z","gameId":"g2"}]},
            "resultStreak":{"win":{"cur":{"v":3},"max":{"v":9,
                "from":{"at":"2023-03-01T00:00:00Z","gameId":"g3"}}}}
        }}"#;
    Mock::given(method("GET"))
        .and(path("/api/user/bobby/perf/blitz"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let stats = client(&server)
        .users()
        .perf_stats("bobby", "blitz")
        .await
        .unwrap();
    assert_eq!(stats.rank, Some(42));
    let stat = stats.stat.unwrap();
    assert_eq!(stat.highest.unwrap().int, Some(2100));
    assert_eq!(stat.count.unwrap().op_avg, Some(1850.5));
    assert_eq!(
        stat.best_wins.unwrap().results[0]
            .op_id
            .as_ref()
            .unwrap()
            .name,
        "X"
    );
    assert_eq!(
        stat.result_streak.unwrap().win.unwrap().max.unwrap().v,
        Some(9)
    );
}

#[tokio::test]
async fn activity_returns_entries() {
    let server = MockServer::start().await;
    let body = r#"[{"interval":{"start":1700000000000,"end":1700086400000},
        "games":{"blitz":{"win":5,"loss":2,"draw":1,"rp":{"before":1800,"after":1815}}},
        "puzzles":{"score":{"win":10,"loss":3,"draw":0,"rp":{"before":2000,"after":2020}}},
        "tournaments":{"nb":2,"best":[{"tournament":{"id":"t1","name":"Weekly"},
            "nbGames":7,"score":15,"rank":3,"rankPercent":10}]},
        "follows":{"in":{"ids":["a"]}}}]"#;
    Mock::given(method("GET"))
        .and(path("/api/user/bobby/activity"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let activity = client(&server).users().activity("bobby").await.unwrap();
    let day = &activity[0];
    assert_eq!(day.interval.start, 1_700_000_000_000);
    assert_eq!(day.games.as_ref().unwrap()["blitz"].win, Some(5));
    assert_eq!(
        day.puzzles
            .as_ref()
            .unwrap()
            .score
            .as_ref()
            .unwrap()
            .rp
            .unwrap()
            .after,
        Some(2020)
    );
    assert_eq!(
        day.tournaments.as_ref().unwrap().best[0].rank_percent,
        Some(10)
    );
    assert!(day.other.contains_key("follows"));
}

#[tokio::test]
async fn leaderboards_returns_top_users_per_perf() {
    let server = MockServer::start().await;
    let body = r#"{"bullet":[{"id":"a","username":"A"}]}"#;
    Mock::given(method("GET"))
        .and(path("/api/player"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let boards = client(&server).users().leaderboards().await.unwrap();
    assert_eq!(boards["bullet"][0].username, "A");
}

#[tokio::test]
async fn notes_returns_list() {
    let server = MockServer::start().await;
    let body = r#"[{"text":"strong player","date":1700000000000}]"#;
    Mock::given(method("GET"))
        .and(path("/api/user/bobby/note"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let notes = client(&server).users().notes("bobby").await.unwrap();
    assert_eq!(notes[0].text.as_deref(), Some("strong player"));
}
