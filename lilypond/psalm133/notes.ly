melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 fis2 e2 d4 a'4 b4 d4 cis4 a4 b2 a2 r2 \break

  % Line 2
  d2 d4 cis4 d2 a2 b4 b4 a4 fis4 g2 fis2 r2 \break

  % Line 3
  a2 fis2 b2 a4 g4 fis2 e2 d2 r2 \break

  % Line 4
  d2 fis4 g4 a2 b2 cis4 a4 b4 cis4 d2 r2 \break

  % Line 5
  d2 cis4 b4 a2 fis2 b4 b4 a4 gis4 a2 r2 \break

  % Line 6
  d,2 a'2 b2 a4 g4 fis2 e2 d1 \bar "|."
}
