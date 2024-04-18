from itertools import product


# https://stackoverflow.com/questions/64867925/python-nested-lists-all-combinations
def nested_product(ls):
    lst_positions = [l for l in ls if isinstance(l, list)]
    for p in product(*lst_positions):
        it = iter(p)
        yield [e if not isinstance(e, list) else [next(it)] for e in ls]


def merge_dict(dictionaries):
    dictionary = {}
    for dict in dictionaries:
        dictionary.update(dict)
    return dictionary


def instance_str_to_dict(unsplit_instance):
    """
    Convert single instance from string to dictionary
    """
    instance = {}
    for axis in unsplit_instance.split(","):
        axis, value = axis.split("=")
        instance[axis] = float(value)
    return sort_instance(instance)


def instances_str_to_list(variable):
    """
    Convert variable string to list of dictionaries
    """
    instances = []
    for unsplit_instance in variable.split(";"):
        instances.append(instance_str_to_dict(unsplit_instance))
    return instances


def instance_dict_to_str(instance):
    """
    Convert single instance from dictionary to string
    """
    axes = []
    for axis, value in sort_instance(instance).items():
        axes.append(f"{axis}={float(value)}")
    return ",".join(axes)


def instances_list_to_str(variable):
    """
    Convert list of dictionaries to string
    """
    return ";".join([instance_dict_to_str(x) for x in variable])


def sort_instance(axis_values):
    """
    Sorts a list of axis values by their axis tag.
    """

    if type(axis_values) is dict:
        return dict(sorted(axis_values.items()))
    elif type(axis_values) is str:
        as_dict = instance_str_to_dict(axis_values)
        return instance_dict_to_str(dict(sorted(as_dict.items())))


def sort_instances(axis_values):
    """
    Sorts a list of axis values by their axis tag.
    """

    if type(axis_values) is list:
        return [sort_instance(x) for x in axis_values]
    elif type(axis_values) is str:
        return ";".join([sort_instance(x) for x in axis_values.split(";")])


def stat_table_combinations(ttFont):
    """
    Returns a list of all combinations of STAT table axis values.
    """

    stat = ttFont["STAT"]

    instances = []

    for i, axis in enumerate(stat.table.DesignAxisRecord.Axis):
        per_axis = []
        for axisValue in stat.table.AxisValueArray.AxisValue:
            if axisValue.AxisIndex == i:
                per_axis.append(
                    {
                        axis.AxisTag: axisValue.Value,
                    }
                )
        instances.append(per_axis)

    for v in list(nested_product(instances)):
        yield sort_instance(merge_dict([x[0] for x in v]))


def fvar_instances(ttFont):
    """
    Returns a list of fvar instances with their axis values.
    """

    fvar = ttFont["fvar"]
    return [sort_instance(instance.coordinates) for instance in fvar.instances]


def combined_axis_locations(ttFont):
    """
    Returns a list of all instances from the STAT table and the fvar table.
    """

    locations = list(stat_table_combinations(ttFont))
    for instance in fvar_instances(ttFont):
        if instance not in locations:
            locations.append(instance)

    return locations


# if __name__ == "__main__":
#     from fontTools.ttLib import TTFont

#     ttFont = TTFont("tests/fonts/RobotoFlex-Var.ttf")
#     # ttFont = TTFont("tests/fonts/Foldit-VariableFont_wght.ttf")

#     mapping = combined_axis_locations(ttFont)
#     print(len(mapping))
#     print(mapping[:5])
