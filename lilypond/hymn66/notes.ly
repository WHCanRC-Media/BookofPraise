melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  fis4 fis4 fis4 fis4. e8 e4 g4 g4 g4 g4 fis2 \break

  % Line 2
  b4 cis4 b4 a4. g8 fis4 e4 fis4 gis4 a2. \break

  % Line 3
  a4 b4 cis4 d4. cis8 b4 a4 g4 fis4 fis4 e2 \break

  % Line 4
  b'4 cis4 d4 d4. a8 a4 fis4 fis4 e4 d2 \break

  % Line 5
  a'4 a4 e4 g8. fis16 fis2 b4 b4 fis4 a8. g16 g2 \break

  % Line 6
  a4 b4 cis4 d4 a4 b4 cis4 d4 b4 a2. \break

  % Line 7
  a4 b4 cis4 d4. cis8 b4 a4 g4 fis4 fis4 e2 \break

  % Line 8
  b'4 cis4 d4 d4. a8 a4 fis4 g4 cis,4 d2. \bar "|."
}
