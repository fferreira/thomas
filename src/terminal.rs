
// Fixme: what's the type for a generic function that does this?
// pub fn innit<'a, I>(a: &'a I) -> Box<dyn Fn(&'a I) -> Option<I> + 'a>
//     where I: PartialEq + Copy {
//     Box::new(move |c| if *c == *a { Some(*c) } else { None })
// }

pub mod unicode {
    use unicode_general_category::{get_general_category, GeneralCategory};

    pub fn innit(a: char) -> Box<dyn Fn(&char) -> Option<char>> {
        Box::new(move |c| if *c == a { Some(*c) } else { None })
    }

    pub fn is_cat(a: GeneralCategory) -> Box<dyn Fn(&char) -> Option<char>> {
        Box::new(move |c| if get_general_category(*c) == a { Some(*c) } else { None })
    }
}