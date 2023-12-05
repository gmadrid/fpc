# FPC

I have a giant image file (generated from a PDF) that contains images of playing cards.
I have been finding it a little impractical in use. I want to create a tool that will:
- find the card images (possibly given hints)
- determine the image bounds for the cards
- add padding, if desired
- enforce a desired aspect ratio on the bounding boxes. (Like 2.5:3.5 for Poker cards.)
- output images at a desired size for all of the cards.
- put rounded corners on the output images
- allow specifying a background color (since my input image is transparent)

## Optional features that are probably useful for debugging

- output a single card
- output just the bounding box (maybe plus some padding)
- maybe show the bounding box outlined in red or some other color.


> __NOTE:__ I cannot include my input playing card image in this project since it is
> a commercial product and not mine to share.


## Notes on test data

Test images are all 72px/in. 

> __NOTE:__ from Illustrator, set up an image of 2.5inx3.5in.  
> Export to PDF: check off `Use Artboards`, and `Screen (72ppi)` for Resolution.

1. circle: a circle centered in a transparent image.
   - image dimensions: 2.5inx3.5in (180x252)
   - circle center: 90ptx126pt
   - circle diameter: 70pt
   - expected bounding box: (55,91)=70x70
2. rect: a rect centered in a transparent image.
   - image dimensions: 2.5inx3.5in (180x252)
   - Rect at 40,66
   - Rect size 100x120
3. circle_border: a circle centered in a transparent box with a black border.
   - image dimensions: 1inx1.25in
   - circle at 21,30
   - circle diameter: 30px
4. grid: a 3x4 grid of images containing geometric shapes with a gray border between them.

## Other random notes.

`2.5/3.5 = 0.71428571428`  
So a card with width 100 will be `100x140`.
