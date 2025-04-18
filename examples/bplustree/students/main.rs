use std::net::IpAddr;
use unionfind::bplustree::BPlusTree;
use unionfind::bplustree::debug::{DebugOptions, print_bplustree};
use uuid::Uuid;

#[derive(Debug)]
enum Gender {
    Male,
    Female,
}

#[derive(Debug)]
struct Student {
    id: Uuid,
    name: String,
    age: i32,
    gender: Gender,
    ip: IpAddr,
}

#[rustfmt::skip]
const MALE_NAMES: [&str; 192] = [
    "Oliver", "Liam", "Ethan", "Mason", "James", "Elijah", "Benjamin", "Lucas", "Alexander", "Jackson", "Daniel", "Matthew", "Sebastian", "Jack", "Michael", "Samuel", "Joseph", "David", "Gabriel", "Carter", "Ryan", "Nathan", "Isaac", "Isaiah", "Hudson", "Theo", "Finn", "Julian", "Zane", "Leo", "Hudson", "Lucas", "Levi", "Milo", "Archer", "Emmett", "Jaxon", "Maximus", "Levi", "Maddox", "Liam", "Isaac", "Aiden", "Jack", "Gabriel", "Elijah", "Eli", "Benjamin", "Ezra", "Caleb", "Ryan", "Max", "Luke", "Theo", "Elliot", "Nash", "Carter", "Everett", "Oliver", "Jackson", "Jaden", "Mason", "Xander", "Grayson", "Wyatt", "Milo", "Hudson", "Cooper", "Jonah", "Riley", "Asher", "Dylan", "Seth", "Easton", "Zander", "Eli", "Zane", "Harrison", "Levi", "Reed", "Nolan", "Emmett", "Finn", "Kai", "Micah", "Xander", "Samuel", "Archer", "Silas", "Isaiah", "Luca", "Adam", "Jameson", "Jaxson", "Owen", "Declan", "Elliot", "Maverick", "Wyatt", "Lachlan", "Hunter", "Tanner", "Sullivan", "Lennox", "Charlie", "Theo", "Ezra", "Jude", "Bennett", "Grayson", "Oliver", "Max", "Dante", "Maverick", "Caden", "Xander", "Isaiah", "Finley", "Micah", "Sebastian", "Samuel", "David", "Jaxon", "Soren", "Lincoln", "Jasper", "Maverick", "Kai", "Lachlan", "Kai", "Sage", "Colton", "Reed", "Levi", "Dax", "Elijah", "Zane", "Maddox", "Asher", "Sawyer", "Everett", "Charlie", "Jonas", "Cole", "Griffin", "Theo", "Miles", "Isaiah", "Jonah", "Theo", "Finn", "Nash", "Jackson", "Seth", "Bennett", "Aiden", "Asher", "Graham", "Liam", "Maddox", "Benjamin", "Maximus", "Caleb", "Samuel", "Blake", "Kai", "Cameron", "Ryder", "Parker", "Harrison", "Nolan", "Maximus", "Hugh", "Caden", "Jonah", "Hudson", "Aidan", "Declan", "Benjamin", "Finn", "Zane", "Hayden", "Maverick", "Hudson", "Nash", "Xander", "Levi", "Zane", "Eli", "David", "Caleb", "Grayson"
];

#[rustfmt::skip]
const FEMALE_NAMES: [&str; 273] = [
    "Emma","Sophia","Ava","Isabella","Mia","Amelia","Harper","Ella","Lily","Charlotte","Aria","Grace","Chloe","Abigail","Scarlett","Zoe","Victoria","Layla","Madison","Nora","Riley","Avery","Leah","Stella","Ella","Luna","Scarlett","Violet","Maya","Mia","Sophie","Sienna","Olivia","Lily","Juliana","Sophia","Sophie","Sienna","Sierra","Avery","Addison","Charlotte","Amaya","Emily","Bella","Sadie","Zoe","Eleanor","Aidan","Lila","Gracie","Madeline","Eliza","Isla","Ava","Emily","Lila","Aiden","Zoe","Charlotte","Addison","Hannah","Aria","Maya","Zoey","Hannah","Sadie","Amelia","Avery","Isabella","Madeline","Lily","Harlow","Luna","Ava","Hannah","Grace","Madison","Victoria","Evelyn","Charlotte","Sophie","Layla","Zoe","Ella","Harper","Abigail","Lydia","Nina","Sophia","Maya","Addison","Adeline","Olivia","Victoria","Isabelle","Ivy","Kinsley","Aubrey","Sophie","Layla","Mia","Riley","Ellie","Addison","Scarlett","Harper","Ava","Mia","Aubrey","Chloe","Violet","Lillian","Ava","Stella","Ivy","Savannah","Gabriella","Delilah","Emily","Madison","Camila","Ella","Adeline","Mackenzie","Hailey","Avery","Arianna","Lily","Natalie","Aubrey","Eden","Paisley","Emery","Amos","Lila","Cora","Sofia","Bella","Grace","Maya","Sierra","Zoe","Kaitlyn","Harper","Norah","Sadie","Addison","Charlotte","Madeline","Lila","Evelyn","Emma","Amelia","Leah","Ava","Lily","Samantha","Ruby","Layla","Zoe","Clara","Victoria","Evelyn","Quinn","Zoey","Sophie","Peyton","Mila","Lillian","Lucy","Ellie","Isla","Leah","Hannah","Stella","Lillian","Alyssa","Ellie","Sienna","Maggie","Emma","Chloe","Sophia","Gabriella","Avery","Chloe","Amelia","Evelyn","Isabella","Violet","Harper","Scarlett","Sophie","Hazel","Leah","Zoe","Lily","Maddie","Sarah","Evelyn","Mya","Adeline","Zoe","Chloe","Mia","Harper","Abigail","Lydia","Ella","Hazel","Ivy","Addison","Charlotte","Layla","Norah","Lilly","Avery","Emily","Ellie","Mia","Madeline","Aubrey","Sarah","Gabrielle","Mia","Zara","Lia","Ava","Layla","Juliana","Hazel","Emily","Lily","Ella","Lydia","Maddie","Madison","Sadie","Emilia","Chloe","Lily","Amara","Isabella","Luna","Sophie","Emily","Audrey","Emma","Hannah","Lydia","Addison","Tessa","Eva","Amos","Avery","Ella","Maya","Lila","Mackenzie","Sophie","Autumn","Stella","Violet","Maddison","Aubrey","Sarah","Maya","Grace","Addison","Isla","Ella","Ruby",
];

#[rustfmt::skip]
const SURNAMES: [&str; 166] = [
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez", "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson", "Thomas", "Taylor", "Moore", "Jackson", "White", "Harris", "Martin", "Thompson", "Robinson", "Clark", "Lewis", "Young", "Walker", "Hall", "Allen", "Scott", "King", "Wright", "Mitchell", "Perez", "Evans", "Collins", "Stewart", "Sanchez", "Morris", "Rogers", "Reed", "Cook", "Morgan", "Bell", "Bailey", "Cooper", "Richardson", "Cox", "Howard", "Ward", "Torres", "Nguyen", "Perez", "Evans", "Baker", "Gonzalez", "Nelson", "Adams", "Graham", "Morris", "Jenkins", "Foster", "King", "Howard", "Russell", "Hunter", "Webb", "Burns", "Cameron", "Simmons", "Hamilton", "Chavez", "Cunningham", "Snyder", "Miller", "Weaver", "Curtis", "Schmidt", "Fowler", "Sullivan", "Curtis", "Franklin", "Campbell", "Gibson", "Glover", "Hughes", "Howard", "Elliott", "Mendoza", "Fox", "Hayes", "Harrison", "Barnes", "Reyes", "Shaw", "Murphy", "Bailey", "Ryan", "Grant", "Duncan", "Harrison", "Bennett", "Ferguson", "Bishop", "Wagner", "Fuller", "Mason", "Jenkins", "Austin", "Glover", "Morris", "Patterson", "Giles", "Vargas", "Graham", "Schneider", "Hicks", "Hanson", "Knight", "Morales", "Garcia", "George", "Dixon", "Alexander", "Harrison", "King", "Harvey", "Stewart", "Jordan", "Carson", "Mills", "Walsh", "Harris", "Barnett", "Miller", "Wallace", "Wells", "Hamilton", "Gonzales", "Ryan", "Daniels", "Burns", "Kennedy", "Jacobs", "Roberts", "Morales", "Ross", "Perry", "Harrison", "Burnett", "Reed", "Ramos", "Johnston", "Jacobs", "Carlson", "Frazier", "Anderson", "Riley", "Diaz", "Miller", "Rodriguez", "Woods", "Martin", "Young", "Riley"
];

fn random_student() -> Student {
    let id = Uuid::new_v4();

    let (mut name, gender) = if rand::random_bool(0.5) {
        let index = rand::random_range(0..MALE_NAMES.len());
        let name = MALE_NAMES[index].to_string();

        (name, Gender::Male)
    } else {
        let index = rand::random_range(0..FEMALE_NAMES.len());
        let name = FEMALE_NAMES[index].to_string();

        (name, Gender::Female)
    };

    let index = rand::random_range(0..SURNAMES.len());
    let surname = SURNAMES[index];
    name.push_str(&format!(" {}", surname));

    let age = rand::random_range(18..=30);

    let ip = IpAddr::from([
        rand::random::<u8>(),
        rand::random::<u8>(),
        rand::random::<u8>(),
        rand::random::<u8>(),
    ]);

    Student {
        id,
        name,
        age,
        gender,
        ip,
    }
}

fn main() {
    let mut students = BPlusTree::new(20);
    let mut names = BPlusTree::new(20);

    for _ in 0..1000 {
        let student = random_student();

        if names.find(&student.name).is_none() {
            names.insert(student.name.clone(), student.id);
            students.insert(student.id, student);
        }
    }

    print_bplustree(&names, DebugOptions::default());
    println!();
    print_bplustree(&students, DebugOptions::default());
    println!();
    println!("students: {} | names: {}", students.size(), names.size());
}
