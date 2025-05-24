use crate::functions::basic::group as get_basic_functions;
use crate::functions::boolean::group as get_boolean_functions;
use crate::functions::list::group as get_list_functions;
use crate::functions::number::group as get_number_functions;
use crate::functions::number_as_string::group as get_nas_functions;
use crate::functions::object::group as get_object_functions;
use crate::functions::process::group as get_exec_functions;
use crate::functions::string::group as get_string_functions;
use crate::functions::time::group as get_time_functions;
use crate::functions::type_group::group as get_type_functions;
use crate::functions::variables::group as get_variable_functions;
use crate::functions_definitions::FunctionsGroup;

pub fn group() -> FunctionsGroup {
    FunctionsGroup::new("functions")
    .add_sub_group(get_basic_functions())
    .add_sub_group(get_type_functions())
    .add_sub_group(get_list_functions())
    .add_sub_group(get_object_functions())
    .add_sub_group(get_number_functions())
    .add_sub_group(get_string_functions())
    .add_sub_group(get_boolean_functions())
    .add_sub_group(get_time_functions())
    .add_sub_group(get_variable_functions())
    .add_sub_group(get_exec_functions())
    .add_sub_group(get_nas_functions())
    .add_description_line("Functions allow one to manipulate the input. The functions format is `(<function-name> <arg0> <arg1> ..)` where `<argN>` are functions or other types of selection.")
    .add_description_line("See additional help for selection for more details.")
    .add_description_line("See additional help with the group name to see the list of available functions in that group.")
    .root()
}
