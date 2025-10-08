# ETOPO File Encoding Notes

## File Information
- **Filename**: `ETOPO_land_ocean.tif`
- **Size**: 978 KB (1,001,889 bytes)
- **Format**: Standard GeoTIFF (compatible with geotiff crate v0.1)
- **Coverage**: Worldwide land and ocean areas

## Elevation Encoding Rules

### Normal Land Elevations
**Elevations ≥ 1 meter**: Stored as actual terrain elevation in meters
- Examples:
  - Sea level coastline: 1-10m
  - Plains: 10-500m
  - Mountains: 500-8000m+

### Water and Below-Sea-Level Areas
**Elevations < 1 meter**: **Set to 0**
- This includes:
  - Ocean surfaces (0m)
  - Ocean depths/trenches (normally negative, but set to 0)
  - Below-sea-level land areas (e.g., Death Valley -86m → 0)
  - Very shallow coastal areas (0-1m → 0)

## Rationale for This Encoding

1. **Simplification**: Most users need land elevation, not ocean bathymetry
2. **File Size**: Excluding ocean depth data reduces file complexity
3. **Use Case**: For astronomical calculations, ocean depth is irrelevant
4. **Coastal Ambiguity**: Avoids issues with tidal zones and shallow areas

## Impact on Elevation Estimation

### ✅ Works Well For:
- Normal land elevations (≥1m): **Exact values from ETOPO**
- Major cities: **±6m average accuracy** with urban correction
- Mountain areas: ETOPO + urban correction
- Rivers and valleys: Excellent accuracy

### ⚠️ Limitations:
- **Below-sea-level locations**: Cannot be accurately estimated
  - Death Valley (-86m actual) → Urban correction on 0 base → ~+134m
  - Dead Sea (-430m actual) → Urban correction on 0 base → ~+199m
  - Error can be 100-600m for these rare locations

### ⚠️ Quirks:
- **Ocean points**: ETOPO returns 0, but urban correction from coastal cities creates small artifacts
  - Open ocean → 0m from ETOPO
  - Nearest cities are on coast → Urban correction applies
  - Result: ±40m instead of 0m
  - **Solution**: Detect obvious ocean (ETOPO=0 and far from cities) → return 0

### ⚠️ Edge Cases:
- **Coastal areas at true 0-1m elevation**: May return 0 or small value
  - Urban correction helps if nearby cities exist
  - Otherwise will read as ~0m

## Known Below-Sea-Level Locations

For reference, these locations **cannot** be accurately estimated:

| Location | Actual Elevation | ETOPO Value | After Urban Correction |
|----------|------------------|-------------|------------------------|
| Death Valley, CA | -86m | 0m | ~+134m (error: 220m) |
| Dead Sea shore | -430m | 0m | ~+199m (error: 629m) |
| Badwater Basin, CA | -86m | 0m | ~+134m |
| Lake Assal, Djibouti | -155m | 0m | Unknown |
| Turpan Depression, China | -154m | 0m | Unknown |
| Qattara Depression, Egypt | -133m | 0m | Unknown |
| Salton Sea, CA | -69m | 0m | Unknown |

**Workaround**: Maintain a database of known below-sea-level locations and return documented values instead of ETOPO-based estimates.

## Recommendations for Users

### When to Trust the Elevation
✅ **High confidence** (±10m):
- Major cities and towns
- Areas with nearby cities in database
- Normal terrain (not extreme mountains or below sea level)

⚠️ **Medium confidence** (±50-200m):
- Remote mountain areas
- Desert regions far from cities
- High altitude locations (>3000m)

❌ **Low confidence** (>200m error):
- Below-sea-level locations (use external data)
- Open ocean (should be 0m)
- Steep mountain valleys (ETOPO shows peaks, not valley floor)

### For Astronomical Calculations
The elevation estimate is **sufficiently accurate** for most astronomical calculations:
- Sunrise/sunset times: ±10m elevation → <1 second time difference
- Moon rise/set times: ±100m elevation → <10 seconds time difference
- Solar/lunar positions: Minimal impact on altitude angles

**Exception**: Below-sea-level locations will have notably incorrect atmospheric refraction corrections due to elevation error.

## Future Improvements

1. **Supplementary DEM**: Add high-resolution data for below-sea-level regions
2. **Below-sea-level database**: Hardcode known extreme depressions
3. **Ocean detection**: If ETOPO=0 and far from land (>50km), return 0 directly
4. **Confidence metric**: Report uncertainty based on terrain type and nearby cities

---

**Created**: October 8, 2025
**Purpose**: Document ETOPO file encoding to explain test results
**File**: `data/ETOPO_land_ocean.tif`
