use serde::{Deserialize, Serialize};
use std::time::Duration;
use thirtyfour::prelude::*;

#[derive(Debug, Serialize)]
struct Problem {
    id: u32,
    name: String,
    acceptance: f32,
    difficulty: String,
    url: String,
    description: String,
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:59890", caps).await?;
    let mut problems: Vec<Problem> = Vec::new();

    for page in 2..69 {
        driver
            .goto(format!("https://leetcode.com/problemset?page={}", page))
            .await?;
        std::thread::sleep(std::time::Duration::from_secs(10));

        let target_elements = driver
            .query(By::Css("[role=\"rowgroup\"] > div "))
            .all_from_selector()
            .await?;
        for target_element in target_elements {
            // each problem being processed
            let target_element_divs = target_element.find_all(By::Css("div")).await?;
            let target_element_divs_name = target_element_divs[1]
                .find(By::Css("div > div > div > div > a"))
                .await?;
            let target_element_divs_difficulty =
                target_element_divs[8].find(By::Css("span")).await?;
            let target_element_divs_acceptance =
                target_element_divs[7].find(By::Css("span")).await?;

            let problem_name_text = target_element_divs_name.text().await?;
            let problem_split: Vec<&str> = problem_name_text.split(".").collect();
            let problem_id: u32 = problem_split[0].parse().expect("Not a valid number");
            let problem_name: &str = &problem_split[1][1..];

            let problem_url_path = target_element_divs_name.attr("href").await?.unwrap();
            let problem_url = format!("https://www.leetcode.com{}", problem_url_path);

            let problem_difficulty = target_element_divs_difficulty.text().await?;

            let problem_acceptance_string = target_element_divs_acceptance.text().await?;
            let problem_acceptance_str = &problem_acceptance_string[0..4];
            let problem_acceptance: f32 = problem_acceptance_str
                .parse()
                .expect("Failed to load acceptance");

            let scraped_problem = Problem {
                id: problem_id,
                name: problem_name.to_string(),
                difficulty: problem_difficulty,
                description: "".to_string(),
                acceptance: problem_acceptance,
                url: problem_url,
            };
            problems.push(scraped_problem);
        }

        for problem in &mut problems {
            let _problem_page = driver.goto(&problem.url).await?;
            std::thread::sleep(Duration::from_secs(8));

            let problem_statement_divs = driver
                .find_all(By::Css(
                    ".flexlayout__layout > div[data-layout-path=\"/ts0/t0\"] > div > div > div",
                ))
                .await?;
            let description = problem_statement_divs[2].text().await?;
            problem.description = description;
        }

        let json = serde_json::to_string(&problems)?;

        println!("{}", json);
    }

    driver.quit().await?;
    Ok(())
}
