pub fn find_difference<'a, T>(vector1: &'a [T], vector2: &'a [T]) -> Vec<&'a T>
where
    T: PartialEq,
{
    let difference: Vec<&'a T> = vector1.iter().filter(|&elem| !vector2.contains(elem)).collect();
    difference
}

pub fn find_intersection<'a, T>(vector1: &'a [T], vector2: &'a [T]) -> Vec<&'a T>
where
    T: PartialEq,
{
    let intersection: Vec<&'a T> = vector1.iter().filter(|&elem| vector2.contains(elem)).collect();
    intersection
}