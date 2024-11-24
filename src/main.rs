use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;
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
    is_premium: bool,
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:59890", caps).await?;
    let mut problems: Vec<Problem> = Vec::new();
        
    // fetch all problems
    for page in 1..69 {
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
            let problem_id: u32 = problem_split[0].parse().unwrap_or(0);
            let problem_name: &str = &problem_split[1][1..];

            let problem_url_path = target_element_divs_name.attr("href").await?.unwrap();
            let problem_url = format!("https://www.leetcode.com{}", problem_url_path);

            let problem_difficulty = target_element_divs_difficulty.text().await?;

            let problem_acceptance_string = target_element_divs_acceptance.text().await?;
            let problem_acceptance_str = &problem_acceptance_string[0..4];
            // let problem_acceptance: f32 = problem_acceptance_str
            //     .parse()
            //     .expect("Failed to load acceptance");

            let problem_acceptance: f32 = problem_acceptance_str.parse().unwrap_or(0.0);

            let scraped_problem = Problem {
                id: problem_id,
                name: problem_name.to_string(),
                difficulty: problem_difficulty,
                description: "".to_string(),
                acceptance: problem_acceptance,
                url: problem_url,
                is_premium: false,
            };
            problems.push(scraped_problem);
        }
    }
    
    // fetch descriptions
    for problem in &mut problems {
        let _problem_page = driver.goto(&problem.url).await?;
        std::thread::sleep(Duration::from_secs(8));
        
        let problem_statement_divs = driver
            .find_all(By::Css(
                ".flexlayout__layout > div[data-layout-path=\"/ts0/t0\"] > div > div > div",
            ))
            .await?;
        
        if problem_statement_divs.len() < 3 {
            continue;
        }
        let description = problem_statement_divs[2].text().await?;
        problem.description = description;
        if problem.description.len() == 0 {
            problem.is_premium = true;
        }
        println!("Problem with {} parsed.", problem.id);
    }

    let json = serde_json::to_string(&problems)?;

    let path = Path::new("leetcode-questions.json");
    let mut file = match File::create(&path) {
        Err(why) => panic!("Error occured coz {}", why),
        Ok(file) => file,
    };

    match file.write_all(json.as_bytes()) {
        Err(why) => println!("Error while writing to the file coz {why}"),
        Ok(_) => println!("Successfully wrote to the file"),
    }

    driver.quit().await?;
    Ok(())
}
