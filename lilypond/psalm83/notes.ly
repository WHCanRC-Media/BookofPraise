melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  b'2 a4 a4 b2 d2 cis4 cis4 b2 r2 \break

  % Line 2
  a2 b4 cis4 d4 cis4 b2 b2 a2 r2 \break

  % Line 3
  b2 d4 cis4 b4 a4 fis4 a4 g2 fis2 r2 \break

  % Line 4
  d2 fis4 g4 a4 fis4 g4 a4 b2 a2 r2 \break

  % Line 5
  fis2 a4 a4 b4 a4 d4 d4 cis2 b2 r2 \break

  % Line 6
  b2 a4 b4 d2 cis2 b4 a4 g2 fis1 \bar "|."
}
