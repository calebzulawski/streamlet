from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="streamlet",
    version="0.1.0",
    rust_extensions=[RustExtension("streamlet.streamlet", binding=Binding.PyO3)],
    packages=["streamlet"],
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
)
