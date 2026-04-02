import struct
import zlib

# CRC calculation function
def crc32_chunk(chunk_type, data):
    import zlib
    crc = 0xffffffff
    for byte in chunk_type + data:
        crc = (zlib.crc32(bytes([byte]), crc) ^ 0xffffffff) ^ 0xffffffff
    return crc & 0xffffffff

png_sig = b'\x89PNG\r\n\x1a\n'

# IHDR chunk with proper CRC
ihdr_type = b'IHDR'
ihdr_data = struct.pack('>IIBBBBB', 1, 1, 8, 6, 0, 0, 0)
ihdr_crc = zlib.crc32(ihdr_type + ihdr_data) & 0xffffffff
ihdr = struct.pack('>I', len(ihdr_data)) + ihdr_type + ihdr_data + struct.pack('>I', ihdr_crc)

# IDAT chunk with filter byte and transparent pixel
idat_type = b'IDAT'
idat_raw = bytes([0, 0, 0, 0, 0])  # filter byte + RGBA(0,0,0,0)
idat_data = zlib.compress(idat_raw)
idat_crc = zlib.crc32(idat_type + idat_data) & 0xffffffff
idat = struct.pack('>I', len(idat_data)) + idat_type + idat_data + struct.pack('>I', idat_crc)

# IEND chunk
iend_type = b'IEND'
iend_crc = zlib.crc32(iend_type) & 0xffffffff
iend = struct.pack('>I', 0) + iend_type + struct.pack('>I', iend_crc)

with open('ui/icons/icon.png', 'wb') as f:
    f.write(png_sig + ihdr + idat + iend)
print('PNG created with proper CRC')

