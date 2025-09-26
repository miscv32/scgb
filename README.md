# scgb
## usage
- `supercoolgb.exe "path_to_ROM"`. Note: supercoolgb expects a file called dmg_boot.bin in the same directory as the executable. 
- Windows build available in Releases tab.
## compilation
- `git clone` this repository
- `cargo build`
  
## current progress

The following games have been tested. There is no audio, or serial support. MBC1 and MBC3 are supported, although without save files or RTC.
One star (\*) means there are minor graphical bugs. Two stars (\*\*) mean the graphical bugs are significant (affect playability). 3 stars  (\*\*\*) means the game crashes or freezes during emulation.
- Tetris
- Dr. Mario
- Kirby's Dream Land
- The Legend of Zelda - Link's Awakening *
- Super Mario Land *
- Pokemon Red *
- Tennis **
- Mr. Nutz **
- Super Mario Land 2 ***
- Wario Land II ***

dmg-acid2 - mostly correct 

<img width="763" height="687" alt="image" src="https://github.com/user-attachments/assets/84c88fb6-6c57-469c-a440-4a76e1838728" />

known issues:
mole on right side of face should not be present
- caused by incorrect object priority for objects with overlapping x-position
  
boot ROM - working
![image](https://github.com/user-attachments/assets/65481835-3ee6-4097-9197-789a2bcc1f0e)
