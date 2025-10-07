use core::borrow::{Borrow, BorrowMut};

use crate::area_serializer::{AreaSerializer, AreaSerializerIterator};
use crate::{interface, AllocBuffer, AreaImgInfo, NoBuffer, Rotation, Run, IT8951};
use crate::memory_converter_settings::MemoryConverterSetting;
use crate::pixel_serializer::{convert_color_to_pixel_iterator, PixelSerializer};

use embedded_graphics_core::{pixelcolor::Gray4, prelude::*, primitives::Rectangle};

impl<IT8951Interface: interface::IT8951Interface> DrawTarget for IT8951<IT8951Interface, Run, NoBuffer> {
    type Color = Gray4;

    type Error = crate::Error;

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let size = self.size();

        self.fill_solid(
            &Rectangle::new(
                Point::zero(),
                Size {
                    width: size.width,
                    height: size.height,
                },
            ),
            color,
        )
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        // only update visible content
        let area = area.intersection(&self.bounding_box());
        // if the area is zero sized, skip drawing
        if area.is_zero_sized() {
            return Ok(());
        }

        let a = AreaSerializer::new(area, color, self.config.max_buffer_size);
        let area_iter = AreaSerializerIterator::new(&a);
        let memory_address = self
            .dev_info
            .as_ref()
            .map(|d| d.memory_address)
            .expect("Dev info not initialized");

        for (area_img_info, buffer) in area_iter {
            self.load_image_area(
                memory_address,
                MemoryConverterSetting {
                    rotation: (&self.config.rotation).into(),
                    ..Default::default()
                },
                &area_img_info,
                buffer,
            )?;
        }
        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let bb = self.bounding_box();
        let iter = convert_color_to_pixel_iterator(area, &bb, colors.into_iter());
        let memory_address = self
            .dev_info
            .as_ref()
            .map(|d| d.memory_address)
            .expect("Dev info not initialized");

        let pixel = PixelSerializer::new(area.intersection(&bb), iter, self.config.max_buffer_size);

        for (area_img_info, buffer) in pixel {
            self.load_image_area(
                memory_address,
                MemoryConverterSetting {
                    rotation: (&self.config.rotation).into(),
                    ..Default::default()
                },
                &area_img_info,
                &buffer,
            )?;
        }
        Ok(())
    }

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics_core::Pixel<Self::Color>>,
    {
        let memory_address = self
            .dev_info
            .as_ref()
            .map(|d| d.memory_address)
            .expect("Dev info not initialized");
        let size = self.size();
        let width = size.width as i32;
        let height = size.height as i32;
        for Pixel(coord, color) in pixels.into_iter() {
            if (coord.x >= 0 && coord.x < width) || (coord.y >= 0 || coord.y < height) {
                let mut data = [0x00, 0x00];

                let value: u8 = color.luma() << ((coord.x % 2) * 4);
                // little endian layout
                // [P3, P2 | P1, P0]
                if coord.x % 4 > 1 {
                    // pixel 2 and 3
                    data[0] = value;
                } else {
                    // pixel 0 and 1
                    data[1] = value;
                }

                self.load_image_area(
                    memory_address,
                    MemoryConverterSetting {
                        rotation: (&self.config.rotation).into(),
                        ..Default::default()
                    },
                    &AreaImgInfo {
                        area_x: coord.x as u16,
                        area_y: coord.y as u16,
                        area_w: 1,
                        area_h: 1,
                    },
                    &data,
                )?;
            }
        }
        Ok(())
    }
}

impl<IT8951Interface: interface::IT8951Interface> DrawTarget for IT8951<IT8951Interface, Run, AllocBuffer> {
    type Color = Gray4;

    type Error = crate::Error;

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let size = self.size();

        self.fill_solid(
            &Rectangle::new(
                Point::zero(),
                Size {
                    width: size.width,
                    height: size.height,
                },
            ),
            color,
        )
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        // only update visible content
        let area = area.intersection(&self.bounding_box());
        // if the area is zero sized, skip drawing
        if area.is_zero_sized() {
            return Ok(());
        }

        let memory_address = self
            .dev_info
            .as_ref()
            .map(|d| d.memory_address)
            .expect("Dev info not initialized");


        self.buffer

        // TODO: only write to the affected area
        self.load_image_area(
            memory_address,
            MemoryConverterSetting {
                rotation: (&self.config.rotation).into(),
                ..Default::default()
            },
            &AreaImgInfo {
                area_x: area.top_left.x as u16,
                area_y: area.top_left.y as u16,
                area_w: area.size.width as u16,
                area_h: area.size.height as u16
            },
            self.buffer.as_buffer(),
        )?;
        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let bb = self.bounding_box();
        let iter = convert_color_to_pixel_iterator(area, &bb, colors.into_iter());
        let memory_address = self
            .dev_info
            .as_ref()
            .map(|d| d.memory_address)
            .expect("Dev info not initialized");

        let pixel = PixelSerializer::new(area.intersection(&bb), iter, self.config.max_buffer_size);

        for (area_img_info, buffer) in pixel {
            self.load_image_area(
                memory_address,
                MemoryConverterSetting {
                    rotation: (&self.config.rotation).into(),
                    ..Default::default()
                },
                &area_img_info,
                &buffer,
            )?;
        }
        Ok(())
    }

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics_core::Pixel<Self::Color>>,
    {
        let memory_address = self
            .dev_info
            .as_ref()
            .map(|d| d.memory_address)
            .expect("Dev info not initialized");
        let size = self.size();
        let width = size.width as i32;
        let height = size.height as i32;
        for Pixel(coord, color) in pixels.into_iter() {
            if (coord.x >= 0 && coord.x < width) || (coord.y >= 0 || coord.y < height) {
                let mut data = [0x00, 0x00];

                let value: u8 = color.luma() << ((coord.x % 2) * 4);
                // little endian layout
                // [P3, P2 | P1, P0]
                if coord.x % 4 > 1 {
                    // pixel 2 and 3
                    data[0] = value;
                } else {
                    // pixel 0 and 1
                    data[1] = value;
                }

                self.load_image_area(
                    memory_address,
                    MemoryConverterSetting {
                        rotation: (&self.config.rotation).into(),
                        ..Default::default()
                    },
                    &AreaImgInfo {
                        area_x: coord.x as u16,
                        area_y: coord.y as u16,
                        area_w: 1,
                        area_h: 1,
                    },
                    &data,
                )?;
            }
        }
        Ok(())
    }
}

impl<IT8951Interface: interface::IT8951Interface, TLocalBuffer: interface::LocalBuffer> OriginDimensions
    for IT8951<IT8951Interface, Run, TLocalBuffer>
{
    fn size(&self) -> Size {
        let dev_info = self.dev_info.as_ref().unwrap();
        let (w, h) = (dev_info.panel_width as u32, dev_info.panel_height as u32);
        let (w, h) = match self.config.rotation {
            Rotation::Rotate0 | Rotation::Rotate180 => (w, h),
            Rotation::Rotate90 | Rotation::Rotate270 => (h, w),
        };
        Size::new(w, h)
    }
}
