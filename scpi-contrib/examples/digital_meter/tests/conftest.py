from subprocess import Popen, PIPE, STDOUT
import pytest
import logging

class Wrapper(object):
    def __init__(self, p: Popen):
        self.p = p
    
    def event(self, cmd):
        self.p.stdin.write(cmd)
        self.p.stdin.flush()
    
    def query(self, cmd) -> str:
        self.event(cmd)
        return self.p.stdout.readline()

@pytest.fixture
def digital_meter():
    p = Popen(['cargo', 'run', '--example', 'digital_meter'], stdout=PIPE, stdin=PIPE, stderr=STDOUT, encoding='utf8')
    # Consume stdout until cargo prints "running ...""
    while True:
        if p.returncode is not None:
            raise Exception("Failed to start digital_meter example")
        s = p.stdout.readline()
        logging.debug(s)
        if s.strip().lower().startswith("running"):
            logging.info("Detected cargo running!")
            break
    return Wrapper(p)