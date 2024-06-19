import csv
import os

with open(os.path.join(os.path.dirname(__file__), "..", "data", "scripts.csv")) as csvfile:
    scripts = list(csv.reader(csvfile))


def get_script_setting(script, setting_name):
    script_column_index = scripts[0].index("script")
    setting_column_index = scripts[0].index(setting_name)

    for row in scripts[1:]:
        if row[script_column_index] == script:
            return row[setting_column_index]
