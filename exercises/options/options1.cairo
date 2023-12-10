// options1.cairo
// Execute `starklings hint options1` or use the `hint` watch subcommand for a hint.



use option::OptionTrait;

// This function returns how much icecream there is left in the fridge.
// If it's before 10PM, there's 5 pieces left. At 10PM, someone eats them
// all, so there'll be no more left :(
fn maybe_icecream(
    time_of_day: usize
) -> Option<usize> { // We use the 24-hour system here, so 10PM is a value of 22 and 12AM is a value of 0
let res : Option<usize> = Option::Some(time_of_day);
// The Option output should gracefully handle cases where time_of_day > 23.
// TODO: Complete the function body - remember to return an Option!
match res {
        Option::Some(val) => {
            if val < 22 {
                return Option::Some(5);
            }else if val > 21 && val < 25 {
                return Option::Some(0);
            }else if val > 24 {
                return Option::None;
            }
        },
        Option::None(_) => {
            return Option::None;
        },
    }
    res
}


#[test]
fn check_icecream() {
    assert(maybe_icecream(9).unwrap() == 5, 'err_1');
    assert(maybe_icecream(10).unwrap() == 5, 'err_2');
    assert(maybe_icecream(23).unwrap() == 0, 'err_3');
    assert(maybe_icecream(22).unwrap() == 0, 'err_4');
    assert(maybe_icecream(25).is_none(), 'err_5');
}

#[test]
fn raw_value() {
    // TODO: Fix this test. How do you get at the value contained in the Option?
    let icecreams : Option<usize> = maybe_icecream(9);
    assert(icecreams.unwrap() == 5, 'err_6');
}
