import os
import digikey
from digikey.v3.productinformation import KeywordSearchRequest
from digikey.v3.productinformation import ManufacturerProductDetailsRequest
import json
import pandas as pd
import time
import pickle


# Declare environment variables
os.environ['DIGIKEY_CLIENT_ID'] = 'iptaw4tZYY14x1NAU6V1s06fDZaXIM6R'
os.environ['DIGIKEY_CLIENT_SECRET'] = 'mhs4hf2AfP5IQOTh'
os.environ['DIGIKEY_CLIENT_SANDBOX'] = 'False'
os.environ['DIGIKEY_STORAGE_PATH'] = '/home/cdsfsmattner/Desktop/rust-hdl_shane/rust-hdl-digikey-py'

# Function to retrieve all of the data from Digikey based on the manufacturer part number
def retrieve_part_digikey(part_num):
    # TODO: Track failures and retry periodically
    try:
        part_num = str(part_num) # Convert part number to a string
        search_request = ManufacturerProductDetailsRequest(manufacturer_product = part_num, record_count=10) # Create the request
        result = digikey.manufacturer_product_details(body=search_request) # Execute the request

        dict = result.to_dict()["product_details"]
        # Sometimes there's more than 1 part returned for a part number search, for now we'll just take the first one
        d = dict[0] 

        
        data = {part_num:{}} # Create buffer to store json data

        # Manufacturing Parameters
        data[part_num]['manufacturer'] = d['manufacturer']['value']
        data[part_num]['manufacturer_part_number'] = d['manufacturer_part_number']
        data[part_num]['detailed_description'] = d['detailed_description']

        # Technical Parameters
        for i in d['parameters']:
            if i['parameter'] == 'Resistance':
                data[part_num]['resistance'] = i['value']
            elif i['parameter'] == 'Tolerance':
                data[part_num]['tolerance'] = i['value']
            elif i['parameter'] == 'Power':
                data[part_num]['power'] = i['value']
            elif i['parameter'] == 'Temperature Coefficient':
                data[part_num]['temp_coeff'] = i['value']
            elif i['parameter'] == 'Operating Temperature':
                data[part_num]['op_temp'] = i['value']
            elif i['parameter'] == 'Package / Case':
                data[part_num]['package'] = i['value']
            elif i['parameter'] == 'Ratings':
                data[part_num]['ratings'] = i['value']
            elif i['parameter'] == 'Size / Dimension':
                data[part_num]['size_area'] = i['value']
            elif i['parameter'] == 'Operating Temperature':
                data[part_num]['op_temp'] = i['value']

        # Ordering Parameters
        data[part_num]['standard_pricing'] = d['standard_pricing'][0]['unit_price']
        data[part_num]['product_status'] = d['product_status']
        data[part_num]['non_stock'] = d['non_stock']
        data[part_num]['quantity_available'] = d['quantity_available']
        data[part_num]['manufacturer_public_quantity'] = d['manufacturer_public_quantity']
        data[part_num]['manufacturer_lead_weeks'] = d['manufacturer_lead_weeks']
        data[part_num]['quantity_on_order'] = d['quantity_on_order']


        # Agency Requirement Parameters
        data[part_num]['ro_hs_status'] = d['ro_hs_status']
        data[part_num]['lead_status'] = d['lead_status']

        # Resources 
        data[part_num]['product_url'] = d['product_url']
        data[part_num]['primary_datasheet'] = d['primary_datasheet']

        return data
    except Exception as e:
        print(str(e))

def get_list_svg(path):
    # Get a list of the parts to convert by looking at the SVG files
    parts_to_convert = []
    seen = set(parts_to_convert)
    print(os.getcwd())
    for file in os.listdir(path):
        if file.endswith(".svg"):
            t = file.split('_')
            t2 = t[0].split('.')
            if t2[0] not in seen:
                seen.add(t2[0])
                parts_to_convert.append(t2[0])
    return parts_to_convert


svg_path = "rust-hdl-digikey-py/symbols"
svg_list = get_list_svg(svg_path)
# Search for the part data, convert it to json, and append it to a list
data_from_parts = []
for p in svg_list:
        d = retrieve_part_digikey(p)
        data_from_parts.append(d)
        time.sleep(1)


# Pickle the results so we can work with the data without sending requests to Digikey


# Saving the objects:
with open('rust-hdl-digikey-py/objs.pkl', 'wb') as f:  # Python 3: open(..., 'wb')
    pickle.dump(data_from_parts, f)

output_json = json.dumps(data_from_parts, indent=4, sort_keys=True)

jsonFile = open("rust-hdl-digikey-py/data.json", "w")
jsonFile.write(output_json)
jsonFile.close()
