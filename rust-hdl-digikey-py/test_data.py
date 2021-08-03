import pickle
import json



with open('rust-hdl-digikey-py/objs.pkl', 'rb') as f:  # Python 3: open(..., 'rb')
    parts_data = pickle.load(f)

# print(parts_data)



output_json = json.dumps(parts_data, indent=4, sort_keys=True)

jsonFile = open("rust-hdl-digikey-py/data.json", "w")
jsonFile.write(output_json)
jsonFile.close()
