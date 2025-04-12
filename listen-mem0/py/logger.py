import logging
import os
import sys
from datetime import datetime


def setup_logger():
    logger = logging.getLogger("app")
    logger.setLevel(os.getenv("LOG_LEVEL", "INFO").upper())

    # Create console handler with formatting
    console_handler = logging.StreamHandler(sys.stdout)
    formatter = logging.Formatter(
        fmt="%(asctime)s [%(levelname)s] %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S"
    )
    console_handler.setFormatter(formatter)
    
    # Add handler to logger
    logger.addHandler(console_handler)
    
    return logger


logger = setup_logger() 