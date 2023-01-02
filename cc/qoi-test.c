#define STB_IMAGE_IMPLEMENTATION
#define STBI_ONLY_PNG
#define STBI_NO_LINEAR
#include "stb_image.h"

#define STB_IMAGE_WRITE_IMPLEMENTATION
#include "stb_image_write.h"

#define QOI_IMPLEMENTATION
#include "qoi.h"

int main(int argc, char **argv) {

	void *pixels = NULL;
	int w, h, channels;
	qoi_desc desc;
	pixels = qoi_read("wikipedia_008.qoi", &desc, 0);
	channels = desc.channels;
	w = desc.width;
	h = desc.height;

	if (pixels == NULL) {
		printf("Couldn't load/decode %s\n", argv[1]);
		exit(1);
	}
	
	printf("QoiDescriptor { width: %d height: %d, channels: Rgb, colorspace: Srgb }",desc.width,desc.height);
	
	int encoded = 0;
	encoded = stbi_write_png("c.png", w, h, channels, pixels, 0);

	if (!encoded) {
		printf("Couldn't write/encode %s\n", argv[2]);
		exit(1);
	}

	free(pixels);
	return 0;
}
