from setuptools import setup, find_packages
import platform
import os

WIN = platform.system() == "Windows"
MAC = platform.system() == "Darwin"
LINUX = platform.system() == "Linux"


install_requires = [
    line
    for line in open(os.path.join(os.path.dirname(__file__), "requirements.txt"), "r").readlines()
    if line.strip() and not line.startswith("#")
]

setup(
    name="fontquant",  # How you named your package folder (MyLib)
    version="0.0.1",  # .post2
    license="apache-2.0",
    # Chose a license from here: https://help.github.com/articles/licensing-a-repository
    description="""Prove and quantify a font's technical quality""",  # Give a short description about your library
    author="Yanone",  # Type in your name
    author_email="post@yanone.de",  # Type in your E-Mail
    url="https://github.com/yanone/fontquant",
    # Provide either the link to your github or to your website
    keywords=["fonts"],  # Keywords that define your package best
    install_requires=install_requires,
    classifiers=[
        "Development Status :: 4 - Beta",
        # Chose either "3 - Alpha", "4 - Beta" or "5 - Production/Stable"
        # as the current state of your package
        "Environment :: Console",
        "Intended Audience :: Developers",  # Define that your audience are developers
        "Topic :: Software Development :: Build Tools",
        "License :: OSI Approved :: Apache Software License",  # Again, pick a license
        "Operating System :: OS Independent",
        "Programming Language :: Python",
        "Programming Language :: Python :: 3",
        "Topic :: Text Processing :: Fonts",
    ],
    entry_points={
        "console_scripts": [
            "fontquant = fontquant.cli:cli",
        ]
    },
    package_dir={"": "Lib"},
    packages=find_packages("Lib"),
    include_package_data=True,
)
