melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  b'2 b4 b4 e,2 b'2 c4 b4 a2 g2 r2 \break

  % Line 2
  a2 b4 a4 g2 fis2 e2 r2 \break

  % Line 3
  b'2 b4 b4 e,2 b'2 c4 b4 a2 g2 r2 \break

  % Line 4
  a2 b4 a4 g2 fis2 e2 r2 \break

  % Line 5
  e2 g4 g4 fis4 fis4 g4 a4 b2 a2 r2 \break

  % Line 6
  b2 c4 b4 a4 a4 g2 r2 \break

  % Line 7
  b2 a4 g4 fis4 d4 e4 fis4 g2 fis2 r2 \break

  % Line 8
  g2 a4 g4 fis4 fis4 e1 \bar "|."
}
