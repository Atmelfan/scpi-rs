from subprocess import Popen
import pytest
from .conftest import Wrapper

def test_idn(digital_meter: Wrapper):
    idn = digital_meter.query("*idn?\n")
    assert(idn == "scpi-rs,digital_voltmeter,0,0\n")


def test_configure(digital_meter: Wrapper):
    # Test default config
    def_conf = digital_meter.query("conf?\n")
    assert(def_conf =='"VOLT:DC AUTO,1.0"\n')

    # Change config to ac voltage
    digital_meter.event("conf:volt:ac 5,0.01\n")
    conf = digital_meter.query("conf?\n")
    assert(conf =='"VOLT:AC 5.0,0.01"\n')

    digital_meter.event("*rst\n")
        # Test default config
    def_conf = digital_meter.query("conf?\n")
    assert(def_conf =='"VOLT:DC AUTO,1.0"\n')

def test_simple_meas(digital_meter: Wrapper):
    # Low level commands
    digital_meter.event("*rst\n")
    digital_meter.event("sens:func \"VOLT:AC\";voltage:ac:range 5V;resolution 0.01V\n")
    res = digital_meter.query("initiate;fetch?\n")
    assert(res == "1.0\n")


