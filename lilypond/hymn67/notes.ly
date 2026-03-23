melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'4 a4 a4 a4 d4 cis4 b2 a2 \break

  % Line 2
  g4 fis4 b4 a4 fis4 g4 e2 \break

  % Line 3
  fis4 fis4 fis4 fis4 b4 a4 a4 gis4 \break

  % Line 4
  a4 b4 cis4 d4 fis,4 gis4 a2 \break

  % Line 5
  d4 cis4 b4 a4 d4 cis4 b4 a4 \break

  % Line 6
  b4 a4 g4 e4 d4 cis4 d2 \bar "|."
}
