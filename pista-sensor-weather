#! /bin/sh

weather_station_id="$1"

weather() {
    curl \
        --silent \
        --show-error \
        -X GET \
        -H "accept: application/vnd.noaa.obs+xml" \
        "https://api.weather.gov/stations/${weather_station_id}/observations/latest?require_qc=false" \
    | hxpipe \
    | awk '
        function path_to_string(path, depth,    p, i) {
            p = ""
            for (i = 1; i <= depth; i++) {
                p = p "/" path[i]
            }
            return p
        }

        function update_node(    paren, name, key, val, path, attr) {
            paren = substr($1, 1, 1)
            name = substr($1, 2, length($1) - 1)
            if (paren == "(") {
                _depth++
                _path[_depth] = name
                XmlPath = path_to_string(_path, _depth)
                for (key in _hxpipe_curr_attrs) {
                    val = _hxpipe_curr_attrs[key]
                    XmlAttr[XmlPath, key] = val
                }
            } else if (paren == ")") {
                delete _hxpipe_curr_attrs
                XmlPayload = ""
                for (key in XmlAttr) {
                    split(key, k, SUBSEP)
                    path = k[1]
                    attr = k[2]
                    if (path == XmlPath) delete XmlAttr[key]
                }
                _depth--
                XmlPath = path_to_string(_path, _depth)
            } else {
                printf("ERROR in input line %d - not a parenthesis: \"%s\"\n", NR, paren) > "/dev/stderr"
                exit 1
            }
        }

        function update_node_attributes(    key, val, s) {
            key = substr($1, 2, length($1))
            val = $0
            s = " +"
            sub("^" $1 s $2 s, "", val)
            _hxpipe_curr_attrs[key] = val
        }

        /^[\(\)]/ {
            update_node()
            next
        }

        /^A/ && $2 == "CDATA" {
            update_node_attributes()
            next
        }

        /^-/ {
            XmlPayload = substr($0, 2, length($0))
        }

        ###########################################################################
        # API:
        #   XmlPath    : string
        #   XmlAttr    : dict : [XmlPath, string] -> string
        #   XmlPayload : string
        ###########################################################################

        XmlPath == "/current_observation/temp_f" {
            printf("%d°F\n", XmlPayload)
            exit 0
        }
    '
}

trap '' PIPE

while :
do
    weather
    sleep "$2"
done
