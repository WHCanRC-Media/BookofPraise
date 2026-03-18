melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 a4 b4 cis2 d2 d4 cis4 b2 a2 r2 \break

  % Line 2
  a2 g4 fis4 a2 e2 g4 fis4 e2 d2 r2 \break

  % Line 3
  d2 a'2 b2 g4 a4 b4 cis4 d2 r2 \break

  % Line 4
  d2 cis4 b4 a4 b2 r8 a2 gis4 a2 r2 \break

  % Line 5
  a2 b4 a4 g2 e2 fis4 g4 a2 r2 \break

  % Line 6
  a2 g2 e2 b'4 b4 a4 g4 fis2 e2 r2 \break

  % Line 7
  a2 b4 cis4 a4 a4 b2 d2 cis2 r2 \break

  % Line 8
  d2 d4 d4 b2 a2 g4 a4 b2 a1 \bar "|."
}
