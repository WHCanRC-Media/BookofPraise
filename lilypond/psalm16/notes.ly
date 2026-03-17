melody = \relative c'' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 d2 cis2 b2 a4 fis4 a4 a4 g4 g4 fis2 r2 \break

  % Line 2
  fis2 e4 d4 a'2 a2 b4 b4 cis4 a4 d2 cis2 r2 \break

  % Line 3
  d2 d4 d4 a2 b2 a4 g2 fis2 e4 fis2 r2 \break

  % Line 4
  d2 e2 g2 fis4 b4 b4 a4 b4 d4 cis2 b2 \break

  % Line 5
  fis2 g4 e4 d2 a'2 g4 fis4 g4 a4 b2 a2 \break

  % Line 6
  fis2 a2 b2 fis4 a4 b4 d4 cis4 b2 ais4 r2 \bar "|."
}
