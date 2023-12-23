# Automate getting your Flight Data

Welcome to my little project! This tool is designed for pilots and aviation enthusiasts.
It automates the process of fetching the latest SimBrief flight plan,
retrieving METAR information for the departure and destination airports,
as well as VATSIM ATIS for both the departure and arrival locations.

## Getting Started

Please follow the steps below to setup your environment:

1. There's a `userdata.json` included with the release.
   You don't need to manually configure this file.
   The GUI supports entering this data:
    - `simBrief_userName`: Your SimBrief username.
    - `api_token`: Your AVWX API Key.
      Visit [AVWX](https://account.avwx.rest/getting-started) to get your API key.
2. Ensure that the `userdata.json` is in the same directory as the executable file.

**Note:** If the program is run via the command line, the `userdata.json`
file should be in the same directory from where the command is run,
not necessarily where the executable file is.

## Running the Project

With the configuration file ready:

- Run the executable file with a double click.
- Alternatively, you can open a terminal,
  navigate to the directory containing the executable file
  and run `./<executable_file_name>`.

Ensure your system meets the essential network requirements
for fetching data from SimBrief, AVWX and Vatsim APIs.

## Functionality

Once up and running:

- The program fetches your latest flight plan from SimBrief.
- It retrieves METAR data for both the departure and destination locations.
- It retrieves the correct VATSIM ATIS (Automated Terminal Information Service)
  for both departure and arrival.

Whether you're a pilot looking to simplify your pre-flight process,
or an aviation enthusiast seeking to automate data retrieval,
I hope my project delivers high value.
