
const IHK_BOUNDARIES: [(u8, f64); 6] = [
    (1, 0.92),
    (2, 0.81),
    (3, 0.67),
    (4, 0.5),
    (5, 0.3),
    (6, 0.0),
];

const TECHNIKER_BOUNDARIES: [(u8, f64); 6] = [
    (1, 0.9),
    (2, 0.7),
    (3, 0.6),
    (4, 0.4),
    (5, 0.3),
    (6, 0.0),
];

const LINEAR_BOUNDARIES: [(u8, f64); 6] = [
    (1, 0.87),
    (2, 0.6),
    (3, 0.47),
    (4, 0.3),
    (5, 0.17),
    (6, 0.0),
];

#[derive(Debug, Clone)]
pub enum GradeScale {
    IHK,
    TECHNIKER,
    LINEAR,
    CUSTOM([(u8, f64); 6])
}

impl GradeScale {
    pub fn values(&self) -> [(u8, f64); 6] {
        match self {
            GradeScale::IHK => IHK_BOUNDARIES,
            GradeScale::TECHNIKER => TECHNIKER_BOUNDARIES,
            GradeScale::LINEAR => LINEAR_BOUNDARIES,
            GradeScale::CUSTOM(values) => *values,
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            GradeScale::IHK => "IHK",
            GradeScale::TECHNIKER => "TECHNIKER",
            GradeScale::LINEAR => "LINEAR",
            GradeScale::CUSTOM(_) => "CUSTOM",
        }
    }
}

// make the IHK scale the default
impl Default for GradeScale {
    fn default() -> Self {
        GradeScale::IHK
    }
}


#[derive(Debug, Clone)]
pub struct GradeRange {
    min: u32,
    max: u32,
}

impl GradeRange {
    pub fn new(min: u32, max: u32) -> Self {
        Self {
            min,
            max,
        }
    } 

    pub fn limits(&self) -> (u32,u32) {
        (self.min, self.max)
    }
}

pub struct Grade {
    value: u32,
    range: GradeRange,
}

impl Grade {
    pub fn new(value: u32, min: u32, max: u32) -> Self {
        Self {
            value,
            range: GradeRange::new(min, max)
        }
    }
    
    pub const fn ref_array(&self) -> [u32; 3] {
        [
            self.value,
            self.range.min,
            self.range.max,
        ]
    }
}


// Grading Calculator
#[derive(Debug, Clone)]
pub struct GradeCalculator {
    pub points: u32,
    pub scale: GradeScale
}

impl Default for GradeCalculator {
    fn default() -> Self {
        Self { points: 100, scale: GradeScale::IHK }
    }
}

impl GradeCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn points(&mut self, points: u32) -> &mut Self {
        //println!("Set max points to {}", points);
        self.points = points;
        self
    }

    pub fn scale(&mut self, scale: GradeScale) -> &mut Self {
        //println!("Use {:?} algorithm", algorithm);
        self.scale = scale;
        self
    }

    pub fn calc(&self) -> Vec<Grade> {
        let mut grades = Vec::new();

        let scale_values = self.scale.values();
        for i in 0..scale_values.len() {
            let grade = scale_values[i].0.clone();
            let min_percentage = scale_values[i].1;
            let max_percentage = if i == 0 {
                1.0 // Maximum percentage for grade 1
            } else {
                scale_values[i - 1].1
            };

            let min_points = (min_percentage * self.points as f64).round();
            let max_points = if i == 0 {
                self.points as f64
            } else {
                (max_percentage * self.points as f64).round() - 1.0
            };

            grades.push(Grade::new(grade as u32, min_points as u32, max_points as u32) );
        }

        grades
    }
}
